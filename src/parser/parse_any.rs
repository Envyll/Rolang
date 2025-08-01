use crate::lexer::Token;
use crate::parser::parser_state::ParserState;
use crate::parser::priority::Priority;
use crate::parser::{ASTExpr, ExprInd, TokenInd};
use crate::push_fn;
use std::cell::Cell;
use std::rc::Rc;

impl<'a, IterT: Iterator<Item = Token>> ParserState<'a, IterT> {
    pub fn try_parse_any(&mut self, min_priority: Priority) -> Option<()> {
        let pr = self.current().unwrap().priority();

        if min_priority == Priority::Path
            || pr.0 == Priority::Close
            || min_priority == Priority::Apply && pr.0 < Priority::Apply
        {
            None?
        }

        let this = self;

        let is_bin = Rc::new(Cell::new(None));
        let is_bin2 = is_bin.clone();
        let ind = this.curr_token_ind;

        push_fn! {this => if is_bin.get().is_some() {
            this.parse_binary(this.expressions_len().prev().unwrap(), min_priority);
        } else {
            this.parse_unary(ind, min_priority);
        };}
        push_fn! {is_bin2 = this.try_parse_semiatomic()}

        Some(())
    }

    pub fn parse_binary_relaxed(&mut self, left: ExprInd, min_priority: Priority) {
        let this = self;
        let pr = this.current().unwrap().priority();
        let ind = this.curr_token_ind;
        let is_anything = Rc::new(Cell::new(None));
        let is_anything2 = is_anything.clone();

        if pr.0 >= min_priority {
            push_fn! {this.parse_binary_relaxed(this.expressions_len().prev().unwrap(), min_priority)}
            push_fn! {this => if is_anything.get().is_some() {
                    *this += ASTExpr::Binary(left, ind, this.expressions_len().prev().unwrap());
                } else {
                    *this += ASTExpr::Empty;
                    *this += ASTExpr::Binary(left, ind, this.expressions_len().prev().unwrap());
                };
            }
            this.advance();
            push_fn! {is_anything2 = this.try_parse_any(pr.1)}
        };
    }

    pub fn parse_binary(&mut self, left: ExprInd, min_priority: Priority) {
        let this = self;

        push_fn! {this.parse_binary_relaxed(this.expressions_len().prev().unwrap(), min_priority)}
        push_fn! {this.parse_apply_or_stop(this.expressions_len().prev().unwrap(),)}
        this.parse_path_or_stop(left)
    }

    pub fn parse_unary(&mut self, ind: TokenInd, min_priority: Priority) {
        let this = self;
        let pr_right = this.current().unwrap().priority().1;

        let is_anything = Rc::new(Cell::new(None));
        let is_anything2 = is_anything.clone();
        if min_priority < pr_right {
            push_fn! {this.parse_binary_relaxed(this.expressions_len().prev().unwrap(), min_priority)}
        }
        push_fn! {this =>
            let expr_len = this.expressions_len();
            if is_anything.get().is_some() {
                *this += ASTExpr::Unary(ind, expr_len.prev().unwrap())
            } else {
                *this += ASTExpr::Simple(ind,)
            };
        }
        this.advance();
        is_anything2.set(this.try_parse_any(pr_right));
    }
}
