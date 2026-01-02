use std::error::Error;
use imstr::string::ImString;
use std::rc::Rc;

pub mod lexer;
pub mod normalizer;
pub mod parser;
pub mod semantics;
pub mod generation;

type ImStrData = Rc<String>;
pub type ImStr = ImString<ImStrData>;
pub type Result<T, E = Box<dyn Error>> = core::result::Result<T, E>;