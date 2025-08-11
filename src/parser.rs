use crate::ImStr;
use crate::lexer::Token;
use crate::normalizer::NormalizedTokenIterator;
use crate::parser::parser_state::ParserState;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Index;
use std::rc::Rc;
use thiserror::Error;

mod parse_any;
mod parse_apply;
mod parse_brackets;
mod parse_path;
pub mod parser_state;
pub mod priority;
pub mod visualize;
pub mod ast_expr_traits;

#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct TokenInd(pub usize);

#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
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
    /// error in most cases
    Empty,
    /// `abc`
    Simple(TokenInd),
    /// `abc::def` `abc::def::ghi` `(A B)::C` `expression::('a := A; 'B := B; ?…)`
    Path(ExprInd, ExprVec),
    /// `abc def …`
    Apply(ExprVec),
    /// `+abc` …
    Unary(TokenInd, ExprInd),
    /// `abc <| def` `abc := def` …
    Binary(ExprInd, TokenInd, ExprInd),
    /// `「abc; def」` `{abc}` `⟨|abc|⟩` `()` `{|abc|}` …
    Braced(TokenInd, ExprVec, TokenInd),
    /// Examples\
    /// `abc or ghi` \
    /// `abc when def : ghi` \
    /// `abc then def :{ghi; …}` \
    /// `abc type def : \n ghi\n …\n` \
    /// Grammar:\
    /// lhs op (pattern `:`)? (rhs| `\n` rhs* `\n` | `{` rhs* `}`)
    Flow(ExprInd, TokenInd, Option<ExprInd>, ExprVec),
    /// Examples\
    /// `return smth` \
    /// `when ?last_arg: {…}`\
    /// `a : b`\
    /// `a := b`\
    /// `a ::= b`
    UnFlow(TokenInd, Option<ExprInd>, ExprVec),
}

#[derive(Clone, Debug)]
pub struct AST {
    pub tokens: Vec<Token>,
    pub expressions: Vec<(TokenInd, ASTExpr)>,
    pub errors: HashMap<ExprInd, Vec<ASTErr>>,
    pub lines: Vec<ImStr>,
}

#[derive(Clone, Debug, Copy, Error)]
pub enum ASTErr {
    #[error("The bracket is unclosed")]
    Unclosed,
    #[error("The bracket is unopened")]
    Unopened,
    #[error("Expected expression")]
    NotAtomic,
    #[error("Unexpected `:`")]
    UnexpectedFlowOp,
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

impl AST{
    pub fn root(&self) -> ExprInd {ExprInd(self.expressions.len() - 1)}
}

type ExprVec = Rc<[ExprInd]>;