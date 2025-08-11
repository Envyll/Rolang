pub mod types;
mod modules_analyze;
mod modules;
mod constants_values;

use crate::ImStr;
use std::collections::HashMap;
use crate::parser::{ExprInd, AST};
use crate::semantics::modules::{ModulesTable};

pub enum SemanticError {}

pub struct SemanticContext {
    pub symbols_table: SymbolsTable,
    pub expr_types: HashMap<Place, types::Type>,
    pub errors: HashMap<Place, SemanticError>,
    pub modules: ModulesTable,
}

pub struct SymbolsTable(pub Vec<HashMap<ImStr, Symbol>>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Symbol {
    pub ty: types::Type,
    pub place: Place,
    //todo
}

#[derive(Debug, Clone, Default)]
struct FileTree(HashMap<Vec<ImStr>, AST>);

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Place{
    pub file: Vec<ImStr>,
    pub expr: ExprInd
}
