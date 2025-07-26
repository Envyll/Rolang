use crate::ImStr;
use crate::lexer::Token;
use crate::normalizer::NormalizedTokenIterator;
use crate::parser::parser_state::ParserState;

mod parse_any;
mod parse_apply;
mod parse_brackets;
mod parse_path;
mod parser_old;
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
    pub errors: Vec<(ExprInd, ASTErr)>,
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
