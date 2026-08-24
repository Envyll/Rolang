use self::analyze::Unit;
use super::{ExpressionDescription, FnInstanceDescr, FnInstanceMeta, FunctionDefinition};
use crate::ImStr;
use crate::parser::{AST, ExprInd};
use crate::semantics::ro_type::RoType;
use derivative::Derivative;
use derive_more::Display;
use intmap::IntMap;
use rpds::{HashTrieMap, List};
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use thiserror::Error;

pub(super) mod analyze;
mod analyze_expr;
pub(super) mod analyze_fn;
mod analyze_match;
mod defer;
mod monomorphize;

#[derive(Debug, Clone)]
pub struct AnalyzedAST {
    pub ast: AST,

    pub errors: Errors,
    pub references: Vec<ExpressionDescription>,

    pub functions: HashMap<ImStr, FunctionDefinition>,
    pub instances: RefCell<HashMap<FnInstanceDescr, FnInstanceMeta>>,
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub(super) struct AnalyzerContext {
    pub(super) ast: AST,

    pub(super) functions: HashMap<ImStr, FunctionDefinition>,

    pub(super) errors: Errors,
    pub(super) references: Vec<ExpressionDescription>,

    #[derivative(Debug = "ignore")]
    tasks: VecDeque<Box<dyn FnOnce(&mut Self) -> Unit>>,
    #[derivative(Debug = "ignore")]
    fn_tasks: HashMap<ImStr, Vec<Box<dyn FnOnce(&mut AnalyzerContext) -> Unit + 'static>>>,
}

#[derive(Debug, Clone, Error)]
pub enum AnalyzeErr {
    #[error("Not yet implemented")]
    Unimplemented,
    #[error("Can't find variable with name {0:?}")]
    UnknownVar(ImStr),
    #[error("Invalid call")]
    BadFnCall,
    #[error("Wrong arguments number (expected: {expected}, got: {got})")]
    FnWrongArgumentNumber { expected: usize, got: usize },
    #[error("Wrong pattern syntax")]
    BadPattern,
    #[error("This pattern can't be used for deconstruction of {expected}")]
    WrongPatternType { expected: RoType },
    #[error("Wrong syntax of match branch (can't find `:`)")]
    BadMatchBranch,
    #[error("Unexpected branch type  (expected: {expected}, got: {got})")]
    WrongMatchBranchType { expected: RoType, got: RoType },
    #[error("Unexpected function argument type (expected: {expected}, got: {got})")]
    WrongArgumentType { expected: RoType, got: RoType },
    #[error("The patterns are not exhaustive")]
    NotFullMatch,
    #[error("A binding with this name already created here ({at})")]
    RepeatedBinding { at: ExprInd },
    #[error("The binding is not declared in all possible cases")]
    NotExhaustiveBinding,
    #[error("This binding has other type in another case (expected: {expected}, got: {got})")]
    UnexpectedBindingType { expected: RoType, got: RoType },
    #[error("Statements must be of unit type (expected: (), got: {got})")]
    WrongStatementType { got: RoType },
    #[error("MustDeclareResultType (todo)")]
    MustDeclareResultType,
    #[error("Output as-expression must not contain definitions ({info:#?})")]
    UnexpectedDeclaration { info: VariablesContext },
    #[error("Declared function's return type is not correct (declared: {declared}, got: {got})")]
    WrongReturnType { declared: RoType, got: RoType },
    #[error("TypeRecursionTooDeep (todo)")]
    TypeRecursionTooDeep,
    #[error("Element {num} does not exist in type {ty}")]
    CantGetThisElement { num: u32, ty: RoType },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BindingReference {
    pub ty: RoType,
    pub expr: ExprInd,
    pub branch: usize,
    pub name: ImStr,
}

#[derive(Debug, Clone)]
struct CurrCtx {
    pub variables: VariablesStack,
    pub curr_fn: ImStr,
}

pub type Errors = IntMap<ExprInd, SmallVec<AnalyzeErr, 1>>;
pub type VariablesContext = HashTrieMap<ImStr, BindingReference>;
pub type VariablesStack = List<VariablesContext>;
type AndThen = Box<dyn FnOnce(&mut AnalyzerContext, RoType, CurrCtx) -> Unit + 'static>;

impl BindingReference {
    pub fn variable(self) -> ExpressionDescription {
        ExpressionDescription::Variable(self)
    }
}
