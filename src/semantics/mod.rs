mod analyzer;
pub mod descriptions;
pub mod patterns;
pub mod ro_type;

pub use self::{
    analyzer::analyze_fn::{FnInstanceDescr, FnInstanceMeta, FunctionDefinition},
    analyzer::{AnalyzeErr, AnalyzedAST, BindingReference, VariablesContext, VariablesStack},
    descriptions::ExpressionDescription,
};

use self::analyzer::AnalyzerContext;
