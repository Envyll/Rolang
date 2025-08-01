use crate::lexer::{Token, TokenType};
use crate::parser::parser_state::ParserState;
use crate::parser::priority::Priority;
use crate::parser::{ASTExpr, ExprInd};
use crate::push_fn;
use std::cell::Cell;
use std::rc::Rc;

impl<'a, IterT: Iterator<Item = Token>> ParserState<'a, IterT> {
    pub fn parse_apply(&mut self, left: ExprInd) {
        self.parse_apply_step(left, vec![], true);
    }

    pub fn parse_apply_or_stop(&mut self, left: ExprInd) {
        self.parse_apply_step(left, vec![], false);
    }

    pub fn try_parse_semiatomic(&mut self) -> Option<()> {
        let this = self;
        let ind = this.curr_token_ind;
        let curr = this.current().unwrap();

        if let (Priority::Path, _) = curr.priority() {
            let is_not_empty = Rc::new(Cell::new(None));
            let is_not_empty2 = is_not_empty.clone();

            push_fn! {this =>
                if is_not_empty.get().is_some() {
                    *this += ASTExpr::Unary(ind, this.expressions_len().prev().unwrap())
                } else {
                    *this += ASTExpr::Simple(ind,)
                };
            }
            this.advance();
            push_fn! {is_not_empty2 = this.try_parse_semiatomic()}
        } else if let (Priority::Apply, _) = curr.priority() {
            let is_path = Rc::new(Cell::new(None));
            let is_path2 = is_path.clone();

            push_fn! {this =>
                if is_path.get().is_some() {
                    this.parse_path_or_stop(this.expressions_len().prev().unwrap());
                } else {
                    this.parse_unary(ind, Priority::Path);
                };
            }
            is_path2.set(this.try_parse_atomic());
        } else {
            None?
        }
        Some(())
    }

    pub fn parse_path_or_stop(&mut self, left: ExprInd) {
        let this = self;
        let curr = this.current().unwrap();

        if let TokenType::ColonColon = curr.ty {
            this.parse_path(left);
        }
    }

    fn parse_apply_step(&mut self, left: ExprInd, mut frags: Vec<ExprInd>, allow_empty: bool) {
        let this = self;

        let is_left = Rc::new(Cell::default());
        let is_left2 = is_left.clone();

        push_fn! {this =>
            if let Some(()) = is_left.get() {
                frags.push(this.expressions_len().prev().unwrap());
                this.parse_apply_step(left, frags, true);
            } else if allow_empty {
                *this += ASTExpr::Apply(left, frags);
            };
        }
        is_left2.set(this.try_parse_semiatomic());
    }
}
