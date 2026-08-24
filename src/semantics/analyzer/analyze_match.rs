use super::analyze::Unit;
use crate::lexer::{BracketType, SymbolType, TokenType};
use crate::parser::{ASTExpr, ExprInd};
use crate::semantics::analyzer::{AndThen, CurrCtx};
use crate::semantics::patterns::{
    BindingInfo, PatternColumnData, PatternColumnOrder, PatternColumnPosition,
    PatternColumnSubposition, PatternTableBuilder,
};
use crate::semantics::ro_type::RoType;
use crate::semantics::{
    AnalyzeErr, AnalyzerContext, BindingReference, ExpressionDescription, VariablesContext,
};
use crate::{ImStr, Multimap};
use itertools::Itertools;
use smallbitvec::sbvec;
use smallvec::smallvec;
use std::cell::RefCell;
use std::clone::Clone;
use std::rc::Rc;

impl AnalyzerContext {
    pub(super) fn analyze_let(
        &mut self,
        let_expr: ExprInd,
        ctx: CurrCtx,
        and_then: AndThen,
    ) -> Unit {
        let ASTExpr::UnFlow(_, l, ref r) = self.ast[let_expr] else {
            unreachable!()
        };
        let r = r.iter().cloned().collect_vec();

        // future : probably make a hint from left part of let
        self.analyze_block(
            r.into_iter(),
            ctx,
            Box::new(move |this, mut r_ty, ctx| {
                let Some(l) = l else {
                    this.errors
                        .insert_value(let_expr, AnalyzeErr::Unimplemented);
                    return and_then(this, RoType::Unit, ctx);
                };

                let mut patterns = PatternTableBuilder::default();

                patterns.push_new();
                this.analyze_pattern(
                    l,
                    &mut patterns,
                    &mut r_ty,
                    &mut PatternColumnPosition::default(),
                    1,
                );

                let ctx_new = this.validate_bindings(&mut patterns, let_expr);

                this.references[let_expr.0] = ExpressionDescription::Let {
                    patterns: patterns.build(),
                    pred_ty: r_ty,
                };

                and_then(
                    this,
                    RoType::Unit,
                    CurrCtx {
                        variables: ctx.variables.push_front(ctx_new),
                        ..ctx
                    },
                )
            }),
        )
    }

    pub(super) fn analyze_match(
        &mut self,
        match_expr: ExprInd,
        ctx: CurrCtx,
        and_then: AndThen,
    ) -> Unit {
        let ASTExpr::Flow(l, _, _, ref r) = self.ast[match_expr] else {
            unreachable!()
        };
        let r = r.clone();

        self.analyze_expr(
            l,
            ctx,
            Box::new(move |this: &mut Self, l_ty: RoType, ctx| {
                let branches = Rc::new((
                    // branch indices
                    RefCell::new(vec![]),
                    // pattern table
                    RefCell::new(PatternTableBuilder::default()),
                ));

                let branches2 = branches.clone();
                let l_ty2 = l_ty.clone();

                this.defer_after_one(
                    RoType::new_unknown(),
                    ctx,
                    r.iter().map(|&brexpr| {
                        let branches = branches.clone();
                        let mut l_ty = l_ty.clone();
                        move |this: &mut Self,
                              ctx: CurrCtx,
                              result_ty: RoType,
                              and_then: AndThen| {
                            let (ref branch_indices, ref patterns) = *branches;

                            if let &ASTExpr::UnFlow(t, Some(m), ref right) = &this.ast[brexpr]
                                && let TokenType::Colon = this.ast[t].ty
                            {
                                branch_indices.borrow_mut().push(brexpr);
                                let patterns = &mut *patterns.borrow_mut();

                                let right = right.clone();

                                patterns.push_new();
                                this.analyze_pattern(
                                    m,
                                    patterns,
                                    &mut l_ty,
                                    &mut PatternColumnPosition::default(),
                                    1,
                                );

                                let vars = this.validate_bindings(patterns, match_expr);

                                this.analyze_block(
                                    right.iter().cloned().collect_vec().into_iter(),
                                    CurrCtx {
                                        variables: ctx.variables.push_front(vars),
                                        ..ctx
                                    },
                                    Box::new(move |this, ty, ctx| {
                                        if !ty.try_make_like(&result_ty) {
                                            this.errors.insert_value(
                                                brexpr,
                                                AnalyzeErr::WrongMatchBranchType {
                                                    expected: result_ty,
                                                    got: ty.clone(),
                                                },
                                            );
                                        };
                                        and_then(this, ty, ctx)
                                    }),
                                )
                            } else {
                                this.errors.insert_value(brexpr, AnalyzeErr::BadMatchBranch);
                                this.analyze_expr(brexpr, ctx, and_then)
                            }
                        }
                    }),
                    move |this, ty, ctx| {
                        this.references[match_expr.0] = ExpressionDescription::Match {
                            branches: branches2.0.take(),
                            patterns: branches2.1.take().build(),
                            ty: ty.clone(),
                            pred_ty: l_ty2,
                        };

                        and_then(this, ty, ctx)
                    },
                )
            }),
        )
    }

    pub(super) fn analyze_pattern(
        &mut self,
        mut pattern_expr: ExprInd,
        patterns: &mut PatternTableBuilder,
        mut ty: &RoType,
        path: &PatternColumnPosition,
        count: usize,
    ) {
        macro_rules! err {
            ($expr:expr) => {
                self.errors.insert_value(pattern_expr, $expr)
            };
        }
        macro_rules! undef {
            () => {{
                err!(AnalyzeErr::Unimplemented);
                return ();
            }};
        }

        'repeat: loop {
            let bind = |name: &ImStr, expr: ExprInd| {
                assert_eq!(patterns.bindings.len(), patterns.branches.len());

                for bind in &mut patterns.bindings[patterns.branches.len() - count..] {
                    if let Some(prev) = bind.insert(
                        name.clone(),
                        BindingInfo {
                            pattern_column_position: PatternColumnOrder {
                                position: path.clone(),
                                priority: 0,
                            },
                            ty: ty.clone(),
                            expr,
                        },
                    ) {
                        self.errors
                            .insert_value(expr, AnalyzeErr::RepeatedBinding { at: prev.expr });
                    };
                }
            };

            match &self.ast[pattern_expr] {
                ASTExpr::Empty => {
                    undef!();
                }
                &ASTExpr::Simple(t) => match &self.ast[t].ty {
                    TokenType::Number { .. } => {
                        let ASTExpr::Simple(t) = self.ast[pattern_expr] else {
                            unreachable!()
                        };
                        let Some(n) = self.analyze_number_literal(t, pattern_expr) else {
                            return;
                        };

                        'conds: {
                            if ty.is_int() {
                                break 'conds;
                            }

                            'l: loop {
                                if let RoType::MaybeUnknown(loc) = ty {
                                    match loc.0.get_or_init(|| RoType::Int) {
                                        RoType::Int => break 'conds,
                                        t @ RoType::MaybeUnknown(_) => {
                                            ty = t;
                                        }
                                        _ => break 'l,
                                    }
                                } else {
                                    break 'l;
                                };
                            }

                            err!(AnalyzeErr::WrongPatternType {
                                expected: ty.clone(),
                            });
                            return;
                        }

                        let PatternTableBuilder {
                            elements,
                            branches,
                            bindings: _bindings,
                        } = patterns;

                        match elements.get_mut(path) {
                            Some(PatternColumnData::Int { values, ignore }) => {
                                assert_eq!(values.len(), branches.len());
                                assert_eq!(ignore.len(), branches.len());
                                for value in values.iter_mut().rev().take(count) {
                                    *value = n.into();
                                }
                                for i in branches.len() - count..branches.len() {
                                    ignore.set(i, false);
                                }
                            }
                            None => {
                                let mut values = smallvec![0; branches.len()];
                                let mut ignore = sbvec![true; branches.len()];

                                for value in values.iter_mut().rev().take(count) {
                                    *value = n.into();
                                }
                                for i in branches.len() - count..branches.len() {
                                    ignore.set(i, false);
                                }
                                patterns.elements.insert(
                                    path.clone(),
                                    PatternColumnData::Int { values, ignore },
                                );
                            }
                            #[allow(unreachable_patterns)]
                            _ => {
                                unreachable!()
                            }
                        };
                    }
                    TokenType::Keyword(kw) if kw == "_" => {}
                    TokenType::Keyword(kw) if kw == "int" => {
                        if !ty.try_make_like(&RoType::Int) {
                            err!(AnalyzeErr::WrongPatternType {
                                expected: ty.clone(),
                            });
                        }
                    }
                    TokenType::Symbol(name, SymbolType::Default) => {
                        ({ bind })(name, pattern_expr);
                    }
                    _ => {
                        undef!();
                    }
                },
                ASTExpr::Unary(_, _) => {
                    undef!();
                }
                &ASTExpr::Binary(l, t, r) => {
                    if let TokenType::Keyword(kw) = &self.ast[t].ty
                        && kw == "as"
                    {
                        self.analyze_pattern(l, patterns, ty, path, count);
                        self.analyze_pattern(r, patterns, ty, path, count);
                    } else {
                        undef!();
                    }
                }
                ASTExpr::Path(_, _) => {
                    undef!();
                }
                ASTExpr::Apply(_) => {
                    undef!();
                }
                &ASTExpr::Braced(l, ref m, _) => match self.ast[l].ty {
                    TokenType::Open(
                        BracketType::Round { with_stick: false },
                        SymbolType::Default,
                    ) => {
                        if let Some(expr) = m.iter().next_back() {
                            pattern_expr = *expr;

                            for &expr in m.iter().skip(1) {
                                self.errors.insert_value(expr, AnalyzeErr::BadPattern);
                            }

                            continue 'repeat;
                        } else {
                            'l: loop {
                                match ty {
                                    RoType::MaybeUnknown(n) => {
                                        ty = n.0.get_or_init(|| RoType::Unit)
                                    }
                                    RoType::Unit => break 'l,
                                    _ => {
                                        err!(AnalyzeErr::WrongPatternType {
                                            expected: ty.clone(),
                                        });
                                        break 'l;
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        undef!();
                    }
                },
                ASTExpr::Flow(_, _, _, _) => {
                    undef!();
                }
                ASTExpr::UnFlow(_, _, _) => {
                    undef!();
                }
                ASTExpr::Tuple(e) => {
                    let e = e.clone();
                    let branches_count_start = patterns.branches.len();

                    let mut left_ty = ty;
                    let mut exprs = e.iter();

                    'l: loop {
                        match left_ty {
                            RoType::Tuple(m) if exprs.len() != 0 => {
                                let already = e.len() - exprs.len();
                                self.analyze_pattern(
                                    *exprs.next().unwrap(),
                                    patterns,
                                    &m.0,
                                    &path.push_back(PatternColumnSubposition::Field(already)),
                                    count + patterns.branches.len() - branches_count_start,
                                );

                                left_ty = &m.1;
                            }
                            RoType::Unit if exprs.len() == 0 => {
                                break 'l;
                            }
                            RoType::MaybeUnknown(n) => {
                                left_ty = n.0.get_or_init(|| {
                                    RoType::new_tuple_of_unknown(e.len(), RoType::Unit)
                                });
                            }
                            _ => {
                                err!(AnalyzeErr::WrongPatternType {
                                    expected: ty.clone(),
                                });
                                return;
                            }
                        }
                    }
                }
                &ASTExpr::Either(ref e) => {
                    let e = e.clone();
                    self.analyze_pattern(*e.iter().next().unwrap(), patterns, ty, path, count);

                    for &expr in e.iter().skip(1) {
                        patterns.clone_last_until(count, path);
                        self.analyze_pattern(expr, patterns, ty, path, count);
                    }
                }
            }

            break;
        }
    }

    #[must_use]
    pub(super) fn validate_bindings(
        &mut self,
        patterns: &mut PatternTableBuilder,
        expr: ExprInd,
    ) -> VariablesContext {
        let &branch = patterns.branches.last().unwrap();

        let [ref mut other @ .., ref mut last] = patterns.bindings[..] else {
            unreachable!()
        };

        let mut new_vars = VariablesContext::default();

        for (name, info) in last.iter_mut() {
            let descr = BindingReference {
                ty: info.ty.clone(),
                expr,
                branch,
                name: name.clone(),
            };

            self.references[info.expr.0] = ExpressionDescription::Variable(descr.clone());

            new_vars.insert_mut(name.clone(), descr);
        }

        other
            .iter_mut()
            .rev()
            .take(
                patterns
                    .branches
                    .iter()
                    .rev()
                    .skip(1)
                    .take_while(|n| **n == branch)
                    .count(),
            )
            .for_each(|bindings| {
                for (name, info) in bindings.iter_mut() {
                    if let Some(other_info) = last.get_mut(name) {
                        if !other_info.ty.try_make_like(&mut info.ty) {
                            self.errors.insert_value(
                                info.expr,
                                AnalyzeErr::UnexpectedBindingType {
                                    expected: other_info.ty.clone(),
                                    got: info.ty.clone(),
                                },
                            );
                        }

                        self.references[info.expr.0] =
                            ExpressionDescription::Variable(new_vars[name].clone());
                    } else {
                        self.errors
                            .insert_value(info.expr, AnalyzeErr::NotExhaustiveBinding);
                    }
                }

                for (name, info) in last.iter_mut() {
                    if !bindings.contains_key(name) {
                        self.errors
                            .insert_value(info.expr, AnalyzeErr::NotExhaustiveBinding);
                    }
                }
            });

        new_vars
    }
}
