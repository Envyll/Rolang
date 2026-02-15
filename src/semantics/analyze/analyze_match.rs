use crate::lexer::{BracketType, SymbolType, TokenType};
use crate::parser::ast_expr_traits::{ExprRef, ExprRefType};
use crate::semantics::patterns::{
    PatternColumnData, PatternColumnPosition, PatternColumnSubposition, PatternTableBuilder,
};
use crate::semantics::types::RoType;
use crate::semantics::{AnalyzeErr, AnalyzerContext, VariableDefinition};
use smallbitvec::sbvec;

impl AnalyzerContext {
    /*
        pub(super) fn analyze_match<'t>(&mut self, match_expr: ExprRef<'t>, exprs: impl Iterator<Item = ExprRef<'t>>) -> RoType {
            let mut last_expr = None;

            let mut branches = vec![];

            let mut ty = RoType::Never;

            for expr in exprs {
                if let ExprRefType::UnFlow(t, Some(m) , r) = expr.ty()
                    && let TokenType::Colon = t.ty {
                    last_expr = Some(m);

                    branches.push((self.analyze_pattern(m), expr.ind() ));

                    let mut last_type = RoType::Never;

                    self.do_in_variables_scope(|this| r.for_each(|r|last_type = this.analyze_expr(r)));

                    if let RoType::Never = ty {ty = last_type};
                } else {
                    self.errors[expr.ind().0] = AnalyzeErr::BadPattern;
                    self.analyze_expr(expr);
                }
            }

            if let Some(m) = last_expr
                && let ExprRefType::Simple(t) = m.ty()
                && let TokenType::Keyword(kw) = &t.ty
                && let "_" = kw.as_str() {
                if let AnalyzeErr::BadPattern = self.errors[m.ind().0] {
                    self.errors[m.ind().0] = AnalyzeErr::None;
                }
            } else {
                self.errors[match_expr.ind().0] = AnalyzeErr::NotFullMatch;
            }

            self.references[match_expr.ind().0] =  VariableDefinition::Match(branches, ty.clone());
            ty
        }

        fn analyze_pattern(&mut self, pattern_expr: ExprRef) -> Pattern {
            match pattern_expr.ty() {
                ExprRefType::Empty => {self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented; Pattern::Bad}
                ExprRefType::Simple(t) => {
                    if let TokenType::Number{..} = t.ty{
                        let n = self.analyze_number_literal(&t, pattern_expr);
                        if let Some(n) = n {
                            Pattern::Number(n)
                        } else {Pattern::Bad}
                    } else {
                        self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented; Pattern::Bad
                    }
                }
                ExprRefType::Unary(_, _) => {self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented; Pattern::Bad}
                ExprRefType::Binary(l, t, r) => {
                    if let TokenType::Operator = t.ty
                        && let "|" = t.slice.as_str() {
                        let right = self.analyze_pattern(r);

                        if let Pattern::Or(mut vec) = right {
                            vec.push(self.analyze_pattern(l));
                            Pattern::Or(vec)
                        } else { Pattern::Or(vec![right, self.analyze_pattern(l), ]) }
                    } else {
                        self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;
                        Pattern::Bad
                    }
                }
                ExprRefType::Path(_, _) => {self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;  Pattern::Bad}
                ExprRefType::Apply(_) => {self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;  Pattern::Bad}
                ExprRefType::Braced(_, _, _) => {self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented; Pattern::Bad}
                ExprRefType::Flow(_, _, _, _) => {self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented; Pattern::Bad}
                ExprRefType::UnFlow(_, _, _) => {self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented; Pattern::Bad}
                ExprRefType::Tuple(_m) => {
                    self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented; Pattern::Bad
                }
            }
        }
    */

    /*
    pub(super) fn analyze_match<'t>(&mut self, match_expr: ExprRef<'t>, exprs: impl Iterator<Item = ExprRef<'t>>) -> RoType {
        let mut branches = vec![];

        let mut ty = RoType::Never;

        for expr in exprs {
            if let ExprRefType::UnFlow(t, Some(m) , r) = expr.ty()
                && let TokenType::Colon = t.ty {

                branches.push( expr.ind() );

                let mut last_type = RoType::Never;

                self.do_in_variables_scope(|this| r.for_each(|r|last_type = this.analyze_expr(r)));

                if let RoType::Never = ty {ty = last_type};
            } else {
                self.errors[expr.ind().0] = AnalyzeErr::BadPattern;
                self.analyze_expr(expr);
            }
        }

        ;; to/do!()
    }

    pub(super) fn analyze_pattern<'e, 'a, 'b>(
        &mut self,
        expr: ExprRef<'e>,
        table : &'b mut PatternTable,
        and_then: &(dyn Fn(&'a mut Self, &'b mut PatternTable) + 'e)
    )  {

        match expr.ty() {
            ExprRefType::Empty => {self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;}
            ExprRefType::Simple(t) => {
                if let TokenType::Number{..} = t.ty{
                    let n = self.analyze_number_literal(&t, expr);
                    if let Some(n) = n {
                        if let Err(e) = self.insert_number_range(
                            table,
                            n, n.wrapping_add(1),
                            and_then
                        ) { self.errors[expr.ind().0] = e; };
                    } else {self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;}
                } else {
                    self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                }
            }
            ExprRefType::Unary(_, _) => {self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;}
            ExprRefType::Binary(l, t, r) => {
                if let TokenType::Operator = t.ty
                    && let "|" = t.slice.as_str() {
                    self.analyze_pattern(l, table, and_then);
                    self.analyze_pattern(r, table, and_then);
                } else {
                    self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                }
            }
            ExprRefType::Path(_, _) => {
                self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::Apply(_) => {
                self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::Braced(l, mut ms, _) => {
                if let "(" = l.slice.as_str()
                    && let Some(m) = ms.next()
                    && let None = ms.next()  {
                    self.analyze_pattern(m, table, and_then);
                } else {
                    self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
                }
            }
            ExprRefType::Flow(_, _, _, _) => {
                self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::UnFlow(_, _, _) => {
                self.errors[expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::Tuple(m) => {
                let mut m = m.rev();

                let Some(f) = m.next() else {
                    let pat = match table {
                        PatternTable::None => {
                            *table = PatternTable::Tuple(Default::default())
                        }
                        PatternTable::Tuple(_) => {}
                        PatternTable::Any(_) => {
                            let PatternTable::Any(p) = mem::replace(table, PatternTable::None) else { unreachable!() };
                            *table = PatternTable::Tuple(p);
                        }
                        _ => {self.errors[expr.ind().0] = AnalyzeErr::Unimplemented; return;},
                    };
                    let PatternTable::Tuple(n) = table else { unreachable!() };
                    and_then(self, table);
                    return;
                };


                let and_then= move |this: &'_ mut Self, table| this.analyze_pattern(f, table, and_then);

                fn run<'a, 'b, 'c>(
                    mut m: impl Iterator<Item = ExprRef<'a>>,
                    b: &'b mut AnalyzerContext,
                    c: &'c mut PatternTable,
                    and_then: &dyn Fn(&'b mut AnalyzerContext, &'c mut PatternTable),
                ) {
                    match m.next() {
                        Some(expr) => {
                            let and_then = |this: &mut AnalyzerContext, table: &mut _| this.analyze_pattern(expr, table, and_then);

                            run(m, b, c, &and_then)
                        },
                        None => and_then(b, c)
                    }
                }


                match table {
                    PatternTable::None => {
                        *table = PatternTable::Tuple(Default::default());
                        let PatternTable::Tuple(n) = table else { unreachable!() };
                        let n = n.as_mut();
                        run(m, self, n, &and_then);
                    }
                    PatternTable::Tuple ( sub) => {

                    }
                    PatternTable::Any(next) => {

                    }

                    _ => self.errors[expr.ind().0] = AnalyzeErr::BadPattern,
                };



            }
        }

    }

    fn insert_number_range<'a, 'b>(
        & mut self,
        table : &'b mut PatternTable,
        start : u64,
        end : u64,
        and_then: &(dyn Fn(& mut Self, &'b mut PatternTable))
    ) -> Result<(), AnalyzeErr> {
        if start == end {
            return Ok(());
        }

        let next_table = match table {
            PatternTable::Any(_) => {
                let mut next_table = BTreeMap::new();
                let PatternTable::Any(next) = mem::replace(table, PatternTable::None) else { unreachable!() };
                next_table.insert(0, *next);
                *table = PatternTable::Number(next_table);
                let PatternTable::Number(next_table) = table else { unreachable!() };
                next_table
            }
            PatternTable::None => {
                let mut next_table = BTreeMap::new();
                next_table.insert(0, Default::default());
                *table = PatternTable::Number(next_table);
                let PatternTable::Number(next_table) = table else { unreachable!() };
                next_table
            }
            PatternTable::Number (next_table) => {
                next_table
            }
            _ => {
                return Err(AnalyzeErr::BadPattern);
            }
        };

        for start in [start, end] {
            let pre_start = next_table.range(..start).next_back().unwrap();

            if *pre_start.0 != start {
                next_table.insert(start, pre_start.1.clone());
            }
        }

        if start < end {
            for t in next_table.range_mut(start..end) {
                and_then(self, t.1)
            }
        } else {
            for t in next_table.range_mut(start..) {
                and_then(self, t.1)
            }
            for t in next_table.range_mut(..end) {
                and_then(self, t.1)
            }
        }

        Ok(())
    }*/

    pub(super) fn analyze_match(&mut self, match_expr: ExprRef) -> RoType {
        let (l, _, _, r) = match_expr.is_flow().unwrap();
        let l_ty = self.analyze_expr(l);

        let mut ty = RoType::Never;
        let mut branches = vec![];

        let mut patterns = PatternTableBuilder::default();

        for brexpr in r {
            if let ExprRefType::UnFlow(t, Some(m), right) = brexpr.ty()
                && let TokenType::Colon = t.ty
            {
                branches.push(brexpr.ind());

                let mut last_type = RoType::Never;
                let mut last_expr_ind = None;

                self.do_in_variables_scope(|this| {
                    patterns.push_new(branches.len() - 1);
                    this.analyze_pattern(
                        m,
                        &mut patterns,
                        &l_ty,
                        &mut PatternColumnPosition::default(),
                        1,
                    );
                    right.for_each(|r| {
                        last_type = this.analyze_expr(r);
                        last_expr_ind = Some(r);
                    })
                });

                if let RoType::Never = ty {
                    ty = last_type
                } else if ty != last_type {
                    self.errors[last_expr_ind.unwrap().ind().0] =
                        AnalyzeErr::WrongMatchBranchType {
                            expected: ty.clone(),
                            got: last_type,
                        };
                };
            } else {
                self.errors[brexpr.ind().0] = AnalyzeErr::BadMatchBranch;
                self.analyze_expr(brexpr);
            }
        }

        self.references[match_expr.ind().0] = VariableDefinition::Match {
            branches,
            patterns: patterns.build(),
            ty: ty.clone(),
            pred_ty: l_ty,
        };

        ty
    }

    fn analyze_pattern(
        &mut self,
        pattern_expr: ExprRef,
        patterns: &mut PatternTableBuilder,
        ty: &RoType,
        path: &mut PatternColumnPosition,
        count: usize,
    ) {
        match pattern_expr.ty() {
            ExprRefType::Empty => {
                self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::Simple(t) => {
                if let TokenType::Number { .. } = t.ty {
                    let Some(n) = self.analyze_number_literal(&t, pattern_expr) else {
                        return;
                    };
                    if !ty.is_int() {
                        self.errors[pattern_expr.ind().0] = AnalyzeErr::WrongPatternType {
                            expected: ty.clone(),
                        };
                        return;
                    }

                    let PatternTableBuilder { elements, branches } = patterns;

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
                            let mut values = vec![0; branches.len()];
                            let mut ignore = sbvec![true; branches.len()];

                            for value in values.iter_mut().rev().take(count) {
                                *value = n.into();
                            }
                            for i in branches.len() - count..branches.len() {
                                ignore.set(i, false);
                            }
                            patterns
                                .elements
                                .insert(path.clone(), PatternColumnData::Int { values, ignore });
                        }
                        #[allow(unreachable_patterns)]
                        _ => {
                            unreachable!()
                        }
                    };
                } else {
                    self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;
                }
            }
            ExprRefType::Unary(_, _) => {
                self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::Binary(_, _, _) => {
                self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::Path(_, _) => {
                self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::Apply(_) => {
                self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::Braced(l, mut m, _) => {
                match l.ty {
                    TokenType::Open(
                        BracketType::Round { with_stick: false },
                        SymbolType::Default,
                    ) => {
                        if let Some(expr) = m.next() {
                            self.analyze_pattern(expr, patterns, ty, path, count);
                        } else {
                            if ty.is_unit() {
                                // do nothing
                            } else {
                                self.errors[pattern_expr.ind().0] = AnalyzeErr::WrongPatternType {
                                    expected: ty.clone(),
                                };
                            }
                        }

                        for expr in m {
                            self.errors[expr.ind().0] = AnalyzeErr::BadPattern;
                        }
                    }
                    _ => {
                        self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;
                    }
                }
            }
            ExprRefType::Flow(_, _, _, _) => {
                self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::UnFlow(_, _, _) => {
                self.errors[pattern_expr.ind().0] = AnalyzeErr::Unimplemented;
            }
            ExprRefType::Tuple(e) => {
                let branches_count_start = patterns.branches.len();
                let RoType::Tuple(m) = ty else {
                    self.errors[pattern_expr.ind().0] = AnalyzeErr::WrongPatternType {
                        expected: ty.clone(),
                    };
                    return;
                };
                if m.len() != e.len() {
                    self.errors[pattern_expr.ind().0] = AnalyzeErr::WrongPatternType {
                        expected: ty.clone(),
                    };
                    return;
                }
                for (i, (expr, ty)) in e.zip(m).enumerate() {
                    path.push(PatternColumnSubposition::Field(i));
                    self.analyze_pattern(
                        expr,
                        patterns,
                        ty,
                        path,
                        count + patterns.branches.len() - branches_count_start,
                    );
                    assert_eq!(path.pop(), Some(PatternColumnSubposition::Field(i)));
                }
            }
            ExprRefType::Either(mut e) => {
                self.analyze_pattern(e.next().unwrap(), patterns, ty, path, count);

                for expr in e {
                    patterns.clone_last_until(count, path);
                    self.analyze_pattern(expr, patterns, ty, path, count);
                }
            }
        }
    }
}
