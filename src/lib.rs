use imstr::string::ImString;
use std::rc::Rc;

pub mod lexer;
pub mod normalizer;
pub mod parser;
pub mod semantics;

type ImStrData = Rc<String>;
type ImStr = ImString<ImStrData>;
