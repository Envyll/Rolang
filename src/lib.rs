use imstr::string::ImString;
use std::rc::Rc;

pub mod generation;
pub mod lexer;
pub mod normalizer;
pub mod parser;
pub mod semantics;

type ImStrData = Rc<String>;
pub type ImStr = ImString<ImStrData>;

pub use anyhow::Result;
pub type AResult<T = ()> = Result<T>;
