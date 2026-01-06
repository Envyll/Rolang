use crate::parser::AST;
use crate::semantics::patterns::Pattern;
pub(crate) use crate::semantics::variables::{FunctionDefinition, VariableDefinition};
use crate::ImStr;
use std::collections::HashMap;

mod analyze;
pub mod types;
pub mod patterns;
pub mod variables;

#[derive(Debug, Clone)]
pub struct AnalyzedAST {
    pub ast: AST,
    
    pub functions: HashMap<ImStr, FunctionDefinition>,
    pub errors: Vec<AnalyzeErr>,
    pub references: Vec<VariableDefinition>,
}

#[derive(Debug, Clone, Default)]
struct AnalyzerContext {
    pub variables: Vec<VariablesContext>,
    pub functions: HashMap<ImStr, FunctionDefinition>,
    
    pub errors: Vec<AnalyzeErr>,
    pub references: Vec<VariableDefinition>,
}

#[derive(Debug, Clone, Default)]
pub enum AnalyzeErr {
    #[default]
    None,
    Unimplemented,
    UnknownVar,
    BadFnCall,
    RepeatedArg,
    BadPattern,
    NotFullMatch,
}



pub type VariablesContext = HashMap<ImStr, VariableDefinition>;
