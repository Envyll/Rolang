use crate::lexer::TokenType;
use crate::parser::ast_expr_traits::{ExprRef, ExprRefType};
use crate::semantics::{AnalyzeErr, AnalyzerContext, VariableDefinition};
use crate::semantics::patterns::Pattern;
use crate::semantics::types::RoType;

impl AnalyzerContext {
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
}