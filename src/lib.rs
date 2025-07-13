extern crate core;

use imstr::string::ImString;
use std::rc::Rc;

pub mod lexer;
mod normalizer;
pub mod parser;

type ImStrData = Rc<String>;
type ImStr = ImString<ImStrData>;
