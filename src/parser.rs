use crate::ImStr;
use crate::lexer::Token;
use crate::normalizer::NormalizedTokenIterator;
use crate::parser::parser_state::ParserState;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Index;

mod parse_any;
mod parse_apply;
mod parse_brackets;
mod parse_path;
pub mod parser_state;
pub mod priority;
pub mod visualize;

#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TokenInd(pub usize);

#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExprInd(pub usize);

impl ExprInd {
    pub fn prev(self) -> Option<Self> {
        Some(Self(self.0.checked_sub(1)?))
    }
}

impl From<usize> for TokenInd {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl From<usize> for ExprInd {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Display for ExprInd {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("[Expr #{}]", self.0))
    }
}

#[derive(Clone, Debug)]
pub enum ASTExpr {
    // error in most cases
    Empty,
    // abc
    Simple(TokenInd),
    // abc::def abc::def::ghi (A B)::C expression::('a := A; 'B := B; ?…)
    Path(ExprInd, Vec<ExprInd>),
    // (abc def…)
    Apply(ExprInd, Vec<ExprInd>),
    // (..) (+abc) …
    Unary(TokenInd, ExprInd),
    // (abc <| def) (abc := def) …
    Binary(ExprInd, TokenInd, ExprInd),
    // 「abc; def」{abc} ⟨abc⟩ () {|abc|} …
    Braced(TokenInd, Vec<ExprInd>, TokenInd),
}

#[derive(Clone, Debug)]
pub struct AST {
    pub tokens: Vec<Token>,
    pub expressions: Vec<(TokenInd, ASTExpr)>,
    pub errors: HashMap<ExprInd, Vec<ASTErr>>,
    pub lines: Vec<ImStr>,
}

#[derive(Clone, Debug, Copy)]
#[repr(u8)]
pub enum ASTErr {
    Unclosed,
    Unopened,
    NotAtomic,
}

impl NormalizedTokenIterator {
    pub fn parse(self) -> AST {
        ParserState::parse(self, ParserState::parse_brackets_or_panic)
    }
}

impl Index<TokenInd> for AST {
    type Output = Token;
    fn index(&self, index: TokenInd) -> &Self::Output {
        &self.tokens[index.0]
    }
}

impl Index<ExprInd> for AST {
    type Output = ASTExpr;
    fn index(&self, index: ExprInd) -> &Self::Output {
        &self.expressions[index.0].1
    }
}
