use std::collections::HashMap;
use derivative::Derivative;
use crate::ImStr;
use crate::parser::{ExprInd, AST};

pub mod types;
mod analyze;

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
}



#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, Hash, PartialEq)]
pub struct FunctionDefinition {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub args: VariablesContext,
    pub expr: ExprInd,
}

#[derive(Debug, Clone, Default, Derivative)]
#[derivative(Eq, Hash, PartialEq)]
pub enum VariableDefinition{
    #[default]
    None,
    Function(ImStr),
    Variable{
        name: ImStr, 
        place: ExprInd,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        n: u32
    },
    Special(ImStr),
    Number(u64),
    Unknown
}

pub type VariablesContext = HashMap<ImStr, VariableDefinition>;