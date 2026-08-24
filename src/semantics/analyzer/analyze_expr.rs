use crate::lexer::{BracketType, SymbolType, TokenType};
use crate::parser::{ASTExpr, ExprInd, TokenInd};
use crate::semantics::analyzer::analyze::Unit;
use crate::semantics::analyzer::{AndThen, CurrCtx};
use crate::semantics::ro_type::{Replacements, RoType};
use crate::semantics::{
    AnalyzeErr, AnalyzerContext, ExpressionDescription, FnInstanceDescr,
};
use crate::Multimap;
use itertools::Itertools;
use std::rc::Rc;

impl AnalyzerContext {
    pub(super) fn analyze_expr<'a>(
        &mut self,
        expr: ExprInd,
        ctx: CurrCtx,
        and_then: AndThen,
    ) -> Unit {
        //dbg!(expr.ind());

        match self.ast[expr] {
            ASTExpr::Empty => self.defer(move |this| and_then(this, RoType::new_unknown(), ctx)),
            ASTExpr::Simple(t) => {
                let ctx2 = ctx.clone();
                let ty = self.analyze_var(t, expr, ctx);
                self.defer(move |this| and_then(this, ty, ctx2))
            }
            ASTExpr::Unary(t, e) => self.defer(move |this: &mut Self| {
                this.analyze_expr(
                    e,
                    ctx,
                    Box::new(move |this: &mut Self, rt, ctx| {
                        this.analyze_call(t, [rt].into(), expr, ctx, and_then)
                    }) as Box<dyn FnOnce(&mut _, _, _) -> _ + 'static>,
                )
            }),
            ASTExpr::Binary(l, t, r) => self.defer_after_all(
                ctx,
                [l, r].into_iter().map(|ind| {
                    move |this: &mut Self, ctx, and_then| this.analyze_expr(ind, ctx, and_then)
                }),
                move |this, types, ctx| this.analyze_call(t, types, expr, ctx, and_then),
            ),
            ASTExpr::Path(_, _) => {
                self.errors.insert_value(expr, AnalyzeErr::Unimplemented);
                self.defer(move |this| and_then(this, RoType::new_unknown(), ctx))
            }
            ASTExpr::Apply(ref n) => {
                let n = n.clone();
                self.defer(move |this| {
                    this.analyze_apply(expr, ctx, n.into_iter().cloned(), and_then)
                })
            }
            ASTExpr::Braced(t, ref n, _) => {
                let n = n.clone();
                self.defer(move |this| match &this.ast[t].ty {
                    TokenType::Open(
                        BracketType::Round { with_stick: false },
                        SymbolType::Default,
                    ) => this.analyze_block(
                        n.into_iter().cloned().collect_vec().into_iter(),
                        ctx,
                        and_then,
                    ),
                    _ => {
                        this.errors.insert_value(expr, AnalyzeErr::Unimplemented);
                        this.analyze_block(
                            n.into_iter().cloned().collect_vec().into_iter(),
                            ctx,
                            and_then,
                        )
                    }
                })
            }
            ASTExpr::Flow(l, t, m, ref _r) => {
                if let TokenType::Keyword(kw) = &self.ast[t].ty
                    && kw == "match"
                    && m.is_none()
                {
                    self.defer(move |this| this.analyze_match(expr, ctx, and_then))
                } else {
                    self.errors.insert_value(expr, AnalyzeErr::Unimplemented);
                    self.defer(move |this| {
                        this.analyze_expr(
                            l,
                            ctx,
                            Box::new(move |this: &mut _, _ty, ctx| {
                                and_then(this, RoType::new_unknown(), ctx)
                            }),
                        )
                    })
                }
            }
            ASTExpr::UnFlow(t, _m, ref _r) => {
                if matches!(self.ast[t].ty, TokenType::ColonEq) {
                    self.defer(move |this| this.analyze_let(expr, ctx, and_then))
                } else {
                    self.errors.insert_value(expr, AnalyzeErr::Unimplemented);
                    self.defer(move |this| and_then(this, RoType::new_unknown(), ctx))
                }
            }
            ASTExpr::Tuple(ref n) => {
                let n = n.clone();

                let fns = n.iter().cloned().map(|e| {
                    move |this: &mut Self, ctx, and_then| this.analyze_expr(e, ctx, and_then)
                });

                self.defer_after_all(ctx, fns, move |this, tys, ctx| {
                    let ty = tys.into_iter().rfold(RoType::Unit, |tail, curr| {
                        RoType::Tuple(Rc::new((curr, tail)))
                    });
                    this.references[expr.0] = ExpressionDescription::Tuple(ty.clone());
                    and_then(this, ty, ctx)
                })
            }
            ASTExpr::Either(ref n) => {
                let _n = n.clone();
                self.errors.insert_value(expr, AnalyzeErr::Unimplemented);
                self.defer(move |this| and_then(this, RoType::new_unknown(), ctx))
            }
        }
    }

    pub(super) fn analyze_number_literal(&mut self, name: TokenInd, expr: ExprInd) -> Option<u64> {
        if let Ok(n) = u64::from_str_radix(self.ast[name].slice.as_str(), 10) {
            self.references[expr.0] = ExpressionDescription::Number(n);
            Some(n)
        } else {
            self.errors.insert_value(expr, AnalyzeErr::Unimplemented);
            None
        }
    }

    fn analyze_var(&mut self, name: TokenInd, expr: ExprInd, ctx: CurrCtx) -> RoType {
        match &self.ast[name].ty {
            TokenType::Symbol(n, SymbolType::Default) => {
                for ee in &ctx.variables {
                    if let Some(def) = ee.get(n) {
                        //dbg!(expr.ind().0);
                        self.references[expr.0] = def.clone().variable();
                        return def.ty.clone();
                    }
                }
                //else
                //dbg!(expr.ind().0);
                self.errors
                    .insert_value(expr, AnalyzeErr::UnknownVar(n.clone()));
                self.references[expr.0] = ExpressionDescription::Unknown;
                RoType::new_unknown()
            }
            TokenType::Number { .. } => {
                self.analyze_number_literal(name, expr);
                RoType::Int
            }
            _ => {
                self.errors.insert_value(expr, AnalyzeErr::Unimplemented);
                RoType::new_unknown()
            }
        }
    }

    fn analyze_call(
        &mut self,
        name: TokenInd,
        args: Box<[RoType]>,
        expr: ExprInd,
        ctx: CurrCtx,
        and_then: AndThen,
    ) -> Unit {
        match (&self.ast[name].ty, self.ast[name].slice.as_str()) {
            (TokenType::Symbol(n, SymbolType::Default), _) => {
                let n = n.clone();
                let Some(c) = self.functions.get(&n) else {
                    self.errors.insert_value(expr, AnalyzeErr::UnknownVar(n));
                    self.references[expr.0] = ExpressionDescription::Unknown;
                    return and_then(self, RoType::new_unknown(), ctx);
                };
                if c.args.len() != args.len() {
                    self.errors.insert_value(
                        expr,
                        AnalyzeErr::FnWrongArgumentNumber {
                            got: args.len(),
                            expected: c.args.len(),
                        },
                    );
                    return and_then(self, RoType::new_unknown(), ctx);
                }
                self.defer_until_fn_done(self.ast[name].slice.clone(), move |this: &mut Self| {
                    let mut replaces = Replacements::default();

                    let result = RoType::new_unknown();

                    this.functions[&n].type_on_exit(&args, &result, &mut replaces);
                    let args = Rc::<[RoType]>::from(args);
                    this.functions.get_mut(&ctx.curr_fn).unwrap().runs.push((
                        FnInstanceDescr {
                            name: n.clone(),
                            args: args.clone(),
                        },
                        result.clone(),
                        expr,
                    ));

                    this.references[expr.0] = ExpressionDescription::Function {
                        name: n.clone(),
                        input: args,
                        output: result.clone(),
                    };

                    and_then(this, result, ctx)
                })
            }
            (TokenType::Operator, "+" | "-" | "*") => {
                self.references[expr.0] =
                    ExpressionDescription::Special(self.ast[name].slice.clone());

                for arg_t in &args {
                    if !arg_t.try_make_like(&RoType::Int) {
                        self.errors.insert_value(
                            expr,
                            AnalyzeErr::WrongArgumentType {
                                expected: RoType::Int,
                                got: arg_t.clone(),
                            },
                        );
                    }
                }
                //dbg!(&args);
                and_then(self, RoType::Int, ctx)
            }
            (TokenType::Number { .. }, _) => {
                let Some(n) = self.analyze_number_literal(name, expr) else {
                    return and_then(self, RoType::new_unknown(), ctx);
                };
                let Ok(n) = u32::try_from(n) else {
                    self.errors.insert_value(expr, AnalyzeErr::BadFnCall);
                    return and_then(self, RoType::new_unknown(), ctx);
                };
                self.references[expr.0] = ExpressionDescription::Member(n);
                let [ty] = &*args else {
                    self.errors.insert_value(expr, AnalyzeErr::BadFnCall);
                    return and_then(self, RoType::new_unknown(), ctx);
                };
                let res = RoType::new_unknown();
                if !ty.try_make_like(&RoType::new_tuple_of_unknown(
                    n as _,
                    RoType::Tuple(Rc::new((res.clone(), RoType::new_unknown()))),
                )) {
                    self.errors.insert_value(
                        expr,
                        AnalyzeErr::CantGetThisElement {
                            num: n,
                            ty: ty.clone(),
                        },
                    );
                    return and_then(self, RoType::new_unknown(), ctx);
                }

                and_then(self, res, ctx)
            }
            _ => {
                self.errors.insert_value(expr, AnalyzeErr::Unimplemented);
                and_then(self, RoType::new_unknown(), ctx)
            }
        }
    }

    fn analyze_apply<'a>(
        &mut self,
        expr: ExprInd,
        ctx: CurrCtx,
        mut n: impl Iterator<Item = ExprInd> + ExactSizeIterator,
        and_then: AndThen,
    ) -> Unit {
        let first = &self.ast[n.next().unwrap()];

        let fns = n
            .map(|ind| move |this: &mut Self, ctx, and_then| this.analyze_expr(ind, ctx, and_then));

        let &ASTExpr::Simple(l) = first else {
            self.errors.insert_value(expr, AnalyzeErr::Unimplemented);

            return self.defer_after_all(ctx, fns, move |this, _types, ctx| {
                and_then(this, RoType::new_unknown(), ctx)
            });
        };

        self.defer_after_all(ctx, fns, move |this, types, ctx| {
            this.analyze_call(l, types, expr, ctx, and_then)
        })
    }
}
