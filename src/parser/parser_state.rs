use crate::lexer::{Lines, Token};
use crate::parser::{AST, ASTErr, ASTExpr, ExprInd, TokenInd};
use std::iter::IntoIterator;
use std::ops::{AddAssign, DivAssign, ShlAssign};

pub struct ParserState<'a, IterT: Iterator<Item = Token>> {
    completion_stack: Vec<(TokenInd, Box<dyn FnOnce(&mut Self) + 'a>)>,
    tokens: Vec<Token>,
    pub curr_token_ind: TokenInd,
    expressions: Vec<(TokenInd, ASTExpr)>,
    errors: Vec<(ExprInd, ASTErr)>,
    tokens_iter: IterT,
}

impl<'a, IterT: Iterator<Item = Token>> ParserState<'a, IterT> {
    pub fn current(&mut self) -> Option<&Token> {
        loop {
            if self.curr_token_ind.0 >= self.tokens.len() {
                self.tokens.push(self.tokens_iter.next()?);
            } else {
                return self.tokens.get(self.curr_token_ind.0);
            }
        }
    }

    pub fn advance(&mut self) {
        self.curr_token_ind.0 += 1;
    }

    pub fn parse(
        tokens_iterable: impl IntoIterator<IntoIter = IterT> + Lines,
        f: impl FnOnce(&mut Self) + 'a,
    ) -> AST {
        let lines = tokens_iterable.lines().to_vec();
        let mut this = Self {
            completion_stack: vec![],
            tokens: vec![],
            tokens_iter: tokens_iterable.into_iter(),
            curr_token_ind: TokenInd(0),
            expressions: vec![],
            errors: Default::default(),
        };

        f(&mut this);

        while let Some((ind, f)) = this.completion_stack.pop() {
            // else cleanup "dead" closures
            if ind <= this.curr_token_ind {
                // cleanup "dead" expressions
                while matches!(
                    this.expressions.last(),
                    Some(&(expr_token_ind, _))
                    if expr_token_ind > this.curr_token_ind
                ) {
                    let curr_expr_ind = ExprInd(this.expressions.len().checked_sub(1).unwrap());
                    // cleanup "dead" errors
                    while matches!(
                        this.errors.last(),
                        Some(&(expr_ind, _))
                        if expr_ind == curr_expr_ind
                    ) {
                        this.errors.pop();
                    }
                    this.expressions.pop();
                }
                // run the last closure
                f(&mut this);
            }
        }

        let Self {
            tokens,
            expressions,
            errors,
            ..
        } = this;

        AST {
            tokens,
            expressions,
            errors,
            lines,
        }
    }

    pub fn expressions_len(&self) -> ExprInd {
        ExprInd(self.expressions.len())
    }
}

impl<'a, IterT: Iterator<Item = Token>, FnT: FnOnce(&mut ParserState<'a, IterT>) + 'a>
    ShlAssign<FnT> for ParserState<'a, IterT>
{
    /// Pushes closure
    fn shl_assign(&mut self, rhs: FnT) {
        self.completion_stack
            .push((self.curr_token_ind, Box::new(rhs)));
    }
}

impl<'a, IterT: Iterator<Item = Token>> AddAssign<ASTExpr> for ParserState<'a, IterT> {
    /// Pushes parsed expression and current index
    fn add_assign(&mut self, rhs: ASTExpr) {
        self.expressions.push((self.curr_token_ind, rhs));
    }
}

impl<'a, IterT: Iterator<Item = Token>> DivAssign<ASTErr> for ParserState<'a, IterT> {
    /// Pushes error for the next expression
    fn div_assign(&mut self, rhs: ASTErr) {
        self.errors.push((ExprInd(self.expressions.len()), rhs));
    }
}

#[macro_export]
macro_rules! push_fn {
    {$this:ident => $($stmt:stmt;)*} => {
        *$this <<= move |$this| {{$($stmt)*};}
    };
    {$this:ident.$f:ident($($args:expr),* $(,)?)} => {
        *$this <<= move |$this| $this.$f($($args),*)
    };
    {$val:path = $this:ident.$f:ident($($args:expr),* $(,)?)} => {
        *$this <<= move |$this| $val.set($this.$f($($args),*))
    };
}
