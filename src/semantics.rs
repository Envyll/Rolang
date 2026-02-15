use crate::parser::AST;
pub(crate) use crate::semantics::variables::{FunctionDefinition, VariableDefinition};
use crate::ImStr;
use std::collections::HashMap;
use crate::semantics::types::RoType;

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
    WrongPatternType{expected: RoType},
    BadMatchBranch,
    WrongMatchBranchType{expected: RoType, got: RoType},
    NotFullMatch,
}



pub type VariablesContext = HashMap<ImStr, VariableDefinition>;
