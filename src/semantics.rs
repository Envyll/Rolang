pub mod types;

use crate::ImStr;
use std::collections::HashMap;

pub enum SemanticError {}

pub struct SemanticContext {
    pub symbols_table: SymbolsTable,
    pub types: Vec<types::Type>,
    pub errors: Vec<SemanticError>,
}

pub struct SymbolsTable(pub Vec<HashMap<ImStr, Symbol>>);

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol {
    ty: types::Type,
    //todo
}
