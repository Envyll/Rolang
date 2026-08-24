use crate::lexer::TokenType;
use crate::parser::{ASTExpr, ExprInd};
use crate::semantics::analyzer::analyze::Unit;
use crate::semantics::analyzer::{AndThen, CurrCtx};
use crate::semantics::patterns::{PatternColumnPosition, PatternTable, PatternTableBuilder};
use crate::semantics::ro_type::{Replacements, RoType};
use crate::semantics::{AnalyzeErr, AnalyzerContext, VariablesContext, VariablesStack};
use crate::{ImStr, Multimap};
use itertools::Itertools;
use std::rc::Rc;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub expr: ExprInd,
    pub args: Vec<RoType>,
    pub arg_patterns: Vec<PatternTable>,
    pub return_type: RoType,
    pub runs: Vec<(FnInstanceDescr, RoType, ExprInd)>,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct FnInstanceDescr {
    pub name: ImStr,
    pub args: Rc<[RoType]>,
}

#[derive(Debug, Clone)]
pub struct FnInstanceMeta {
    pub ty: RoType,
    pub replacements: Replacements,
}

impl AnalyzerContext {
    pub(super) fn analyze_fn_declaration(&mut self, expr: ExprInd) -> Option<Unit> {
        let (t, left, _) = self.ast.at(expr).is_un_flow()?;
        let left = left?;
        let TokenType::ColonEq = t.ty else { None? };

        let return_type;

        // if the function has explicit exit type
        let left = if let &ASTExpr::Binary(l, t, r) = left.expr()
            && let TokenType::Keyword(w) = &self.ast[t].ty
            && w == "as"
        {
            let mut table = PatternTableBuilder::default();
            table.push_new();
            let mut ty = RoType::new_unknown();
            self.analyze_pattern(r, &mut table, &mut ty, &mut PatternColumnPosition::new(), 1);

            let vars = self.validate_bindings(&mut table, r);

            if !vars.is_empty() {
                self.errors
                    .insert_value(r, AnalyzeErr::UnexpectedDeclaration { info: vars });
            }

            return_type = ty;
            self.ast.at(l)
        } else {
            return_type = RoType::new_unknown();
            left
        };

        let ASTExpr::Apply(arg_exprs) = left.expr() else {
            self.errors
                .insert_value(left.ind(), AnalyzeErr::Unimplemented);
            None?
        };
        let &fst = arg_exprs.first().unwrap();

        let Some(n) = self.ast.at(fst).is_simple_default_symbol() else {
            self.errors.insert_value(fst, AnalyzeErr::Unimplemented);
            None?
        };
        let name = n.clone();

        let ind = expr;

        let args = (0..arg_exprs.len() - 1)
            .map(|_| RoType::new_unknown())
            .collect_vec();
        let mut arg_patterns = vec![];
        let mut arg_ctx = VariablesContext::new();

        let arg_exprs = arg_exprs.clone();
        arg_exprs
            .iter()
            .skip(1)
            .zip(&args)
            .for_each(|(&arg_expr, arg)| {
                let mut patterns = PatternTableBuilder::default();
                patterns.push_new();

                self.analyze_pattern(
                    arg_expr,
                    &mut patterns,
                    arg,
                    &mut PatternColumnPosition::default(),
                    1,
                );

                let args_new = self.validate_bindings(&mut patterns, ind);

                for (name, arg) in &args_new {
                    if arg_ctx.contains_key(name) {
                        self.errors
                            .insert_value(arg.expr, AnalyzeErr::RepeatedBinding { at: arg.expr });
                        continue;
                    };

                    arg_ctx.insert_mut(name.clone(), arg.clone());
                }

                arg_patterns.push(patterns.build());
            });

        self.functions.insert(
            name.clone(),
            FunctionDefinition {
                expr: ind,
                return_type,
                args,
                arg_patterns,
                runs: Vec::new(),
            },
        );
        self.fn_tasks.insert(name.clone(), vec![]);

        Some(self.defer(|this| {
            this.analyze_fn_definition(CurrCtx {
                curr_fn: name,
                variables: VariablesStack::new().push_front(arg_ctx),
            })
        }))
    }

    pub(super) fn analyze_fn_definition(&mut self, ctx: CurrCtx) -> Unit {
        let def = &self.functions[&ctx.curr_fn];
        let ASTExpr::UnFlow(_, _, right) = &self.ast[def.expr] else {
            unreachable!()
        };
        let right = right.clone();
        let last = *right.last().unwrap();

        self.analyze_block(
            (0..right.len()).map(move |i| right[i]),
            ctx,
            Box::new(move |this, mut ty, ctx| {
                if !this.functions[&ctx.curr_fn]
                    .return_type
                    .try_make_like(&mut ty)
                {
                    this.errors.insert_value(
                        last,
                        AnalyzeErr::WrongReturnType {
                            declared: this.functions[&ctx.curr_fn].return_type.clone(),
                            got: ty.clone(),
                        },
                    )
                }

                for and_then in this.fn_tasks.remove(&ctx.curr_fn).into_iter().flatten() {
                    let _ = this.defer(|this| and_then(this));
                }
                Unit
            }),
        )
    }

    pub(super) fn analyze_block(
        &mut self,
        right: impl Iterator<Item = ExprInd> + ExactSizeIterator + 'static,
        ctx: CurrCtx,
        and_then: AndThen,
    ) -> Unit {
        self.defer(move |this| {
            this._analyze_block_internal(
                right,
                ctx.clone(),
                ctx.variables.len(),
                RoType::Unit,
                ExprInd(-1 as _), //never reachable (i hope)
                and_then,
            )
        })
    }

    fn _analyze_block_internal(
        &mut self,
        mut right: impl Iterator<Item = ExprInd> + ExactSizeIterator + 'static,
        ctx: CurrCtx,
        ctx_orig_len: usize,
        default_ty: RoType,
        prev: ExprInd,
        and_then: AndThen,
    ) -> Unit {
        let Some(next) = right.next() else {
            let mut res_ctx = ctx;
            // cut off newly declared variables
            while res_ctx.variables.len() != ctx_orig_len {
                if !res_ctx.variables.drop_first_mut() {
                    panic!()
                }
            }

            return and_then(self, default_ty, res_ctx);
        };

        if !default_ty.try_make_like(&RoType::Unit) {
            self.errors.insert_value(
                prev,
                AnalyzeErr::WrongStatementType {
                    got: default_ty.clone(),
                },
            );
        }

        self.analyze_expr(
            next,
            ctx,
            Box::new(move |this: &mut Self, ty: RoType, ctx| {
                this._analyze_block_internal(right, ctx, ctx_orig_len, ty, next, and_then)
            }),
        )
    }
}

impl FunctionDefinition {
    pub(super) fn type_on_exit(
        &self,
        args: &[RoType],
        result: &RoType,
        replaces: &mut Replacements,
    ) {
        self.args
            .iter()
            .chain([&self.return_type])
            .map(|patt| patt.deep_clone(replaces))
            .zip_eq(args.iter().chain([result]))
            .for_each(|(patt, arg)| {
                if !patt.try_make_like(arg) {
                    unimplemented!()
                }
            });
    }
}
