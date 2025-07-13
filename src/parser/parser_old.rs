/*use std::rc::Rc;
use imstr::data::Data;
use crate::lexer::{Token, TokenType};
use crate::parser::{ASTErr, ASTExpr, ASTStmt, Definition, FinExpr, OptExpr, ParseAndThen, ParseErr, Parser, Path, Priority, RcExpr, VecStmt, AST};
use crate::parser::ASTExpr::{BinaryOp, Simple, UnaryOp};
use crate::parser::priority::AssocDir;

pub struct Parser (
    Vec<Token>
);

impl NormalizedTokenIterator {
    pub fn parser(self) -> Parser {
        Parser(self.collect())
    }
}

macro_rules! and_then {
    {$curr:pat_param, $right:pat_param => $fin:expr} => {Box::new(move |$curr, $right|$fin)};
}


type ParseAndThen<'a> = Box<dyn FnOnce(&mut usize, ASTExpr) -> FinExpr + 'a>;

impl Parser {

    #[allow(deprecated)]
    pub fn run(self) -> AST {
        let mut val = 0;
        let res = self.parse_some_expr(&mut val, None, Priority::Path, Box::new(identity)).0;

        if val != self.0.len() {
            todo!("unopened brace error handle")
        }

        AST(self.0, res)
    }

    fn parse_op_expr(&self
                     , curr: &mut usize
                     , left: OptExpr
                     , min_priority: Priority
                     , and_then: ParseAndThen
    ) -> FinExpr{
        let start = *curr;
        let start_tok = &self.0[start];

        let start_priority = start_tok.priority();

        if start_priority < min_priority
        || start_priority == min_priority && start_priority.assoc_dir() == AssocDir::Left {
            if let Some(left) = left {
                return and_then(curr, left);
            } else {
                *curr += 1;
                return self.parse_some_expr(curr, None, start_priority, and_then!{curr, right =>
                and_then(curr, UnaryOp(start.into(), right.into()))
            }); }
        }

        *curr += 1;
        self.parse_some_expr(curr, None, start_priority, and_then!{curr, right => {
        let res = if let Some(left) = left {
            BinaryOp(left.into(), start.into(), right.into())
        } else {
            UnaryOp(start.into(), right.into())
        };

        self.parse_some_expr(curr, Some(res), min_priority, and_then)
    }})}

    fn parse_braces(&self
                    , curr: &mut usize
                    , and_then: ParseAndThen
    ) -> FinExpr {
        let start = *curr;
        *curr += 1;

        self.parse_some_expr(curr, None, Priority::Close, and_then!{curr, expr => {
        let end = *curr;

        if *curr != self.0.len() {
            *curr += 1;
            and_then(curr, ASTExpr::Braced(start.into(), expr.into(), end.into()))
        }
        else {
            and_then(curr, ASTExpr::Error(ASTExpr::UnaryOp(start.into(), expr.into()).into(), ASTErr::Unclosed))
        }
    }})}

    fn parse_path(&self
                  , curr: &mut usize
                  , left: ASTExpr
                  , and_then: ParseAndThen
    ) -> FinExpr {
        let TokenType::ColonColon = self.0[*curr].ty else {return and_then(curr, left)};



        todo!()
    }

    fn parse_seq(&self
                 , curr: &mut usize
                 , left: OptExpr
                 , and_then: ParseAndThen
    ) -> FinExpr {
        todo!()
    }

    fn parse_tuple(&self
                   , curr: &mut usize
                   , left: ASTExpr
                   , and_then: ParseAndThen
    ) -> FinExpr {
        todo!()
    }

    fn parse_some_expr(&self
                       , curr: &mut usize
                       , left: OptExpr
                       , min_priority: Priority
                       , and_then: ParseAndThen
    ) -> FinExpr {
        let start = *curr;
        let start_tok = &self.0[start];

        if *curr == self.0.len() {return and_then(curr, left.unwrap_or_default());};

        match &start_tok.ty {
            TokenType::Open => self.parse_braces(curr, and_then!{curr, right =>
                self.parse_apply(curr, left, right, min_priority, and_then)
            }),
            TokenType::ColonColon => self.parse_path(curr, Default::default(), and_then!{curr, right =>
                self.parse_apply(curr, left, right, min_priority, and_then)
            }),
            TokenType::Close => and_then(curr, left.unwrap_or_default()),
            TokenType::Semicolon if min_priority < Priority::Comma => self.parse_seq(curr, left, and_then),
            TokenType::Comma if min_priority < Priority::Comma => self.parse_tuple(curr, left.unwrap_or_default(), and_then),
            _ => match start_tok.priority() {
                Priority::Apply => {*curr += 1; self.parse_apply(curr, left, Simple(start.into()), min_priority, and_then)},

                _ => self.parse_op_expr(curr, left, min_priority, and_then),
            }
        }
    }

    fn parse_apply(&self
                   , curr: &mut usize
                   , left: OptExpr
                   , right: ASTExpr
                   , min_priority: Priority
                   , and_then: ParseAndThen
    ) -> FinExpr {
        if *curr == self.0.len() {return and_then(curr, right);};

        self.parse_path(curr, right, and_then!{ curr, right => {

        let Some(mut left) = left else {
            return self.parse_some_expr(curr, Some(right), min_priority, and_then);
        };

        if let ASTExpr::Apply(_, mid) = &mut left {
            mid.push(right.into())
        } else {
            left = ASTExpr::Apply(left.into(), vec![right.into()])
        }
        self.parse_some_expr(curr, Some(left), min_priority, and_then)
    }})}
}

#[deprecated(note = "please rewrite with closures whenever possible")]
fn identity(curr: &mut usize, left: ASTExpr) -> FinExpr {
    (left,)
}*/
