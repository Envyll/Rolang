use crate::Multimap;
use crate::parser::{AST, ASTExpr};
use crate::semantics::{AnalyzeErr, AnalyzedAST, AnalyzerContext};

impl AnalyzedAST {
    pub fn analyze(ast: AST) -> Self {
        let mut this = AnalyzerContext {
            references: vec![Default::default(); ast.expressions.len()],
            ast,
            ..AnalyzerContext::default()
        };

        this.analyze_top_level_definitions();

        let instances = this.monomorphize().into();

        let AnalyzerContext {
            functions,
            errors,
            references,
            tasks,
            ast,
            fn_tasks,
        } = this;

        assert!(tasks.is_empty());
        assert!(fn_tasks.is_empty());

        Self {
            ast,
            functions,
            errors,
            references,
            instances,
        }
    }
}

//pub(super) type ResultFn = Box<dyn FnOnce(&mut AnalyzerContext, ExprInd, RoType) -> Unit + 'static>;

//for check
#[must_use]
pub(super) struct Unit;

impl AnalyzerContext {
    fn analyze_top_level_definitions(&mut self) {
        let &ASTExpr::Braced(_, ref inds, _) = self.ast.last().expr() else {
            unreachable!()
        };
        let inds = inds.clone();

        for &ind in inds.iter() {
            if self.analyze_fn_declaration(ind).is_none() {
                self.errors.insert_value(ind, AnalyzeErr::Unimplemented);
            }
        }

        'l: loop {
            while let Some(f) = self.tasks.pop_front() {
                let _ = f(self);
            }
            if let Some(some_fn) = self.fn_tasks.keys().next().cloned()
                && let Some(vec) = self.fn_tasks.remove(&some_fn)
            {
                for f in vec {
                    let _ = f(self);
                }
                continue 'l;
            }
            break 'l;
        }
    }
}
