use crate::lexer::Token;
use crate::parser::parser_state::ParserState;
use crate::parser::priority::{Priority, TokenClass};
use crate::parser::ASTErr::UnexpectedComma;
use crate::parser::{/*ASTErr,*/ ASTExpr, ExprInd, ExprVec, TokenInd};
use std::cell::Cell;
use std::rc::Rc;

impl<'a, IterT: Iterator<Item = Token>> ParserState<'a, IterT> {
    pub fn get_curr_token_class_and_priority(&mut self) -> (TokenClass, (Priority, Priority)) {
        let curr = self.current().unwrap();
        (curr.class(), curr.priority())
    }


    pub fn try_parse_any(&mut self, min_priority: Priority, was_flow: bool, allow_prev_unary_right: bool) -> Option<()> {
        let pr = self.current().unwrap().priority();

        if min_priority == Priority::Path
            || pr.0 == Priority::Close
            || min_priority == Priority::Apply && pr.0 < Priority::Apply
            || allow_prev_unary_right && pr.0 < min_priority
        {
            None?
        }

        let this = self;

        let is_bin = Rc::new(Cell::new(None));
        let is_bin2 = is_bin.clone();
        let ind = this.curr_token_ind;

        *this <<= move |this| {
            if is_bin.get().is_some() {
                if !was_flow || this.is_prev_flow_br().is_none() {
                    this.parse_binary(this.expressions_len().prev().unwrap(), min_priority);
                }
            } else {
                this.parse_unary(ind, min_priority);
            };
        };
        is_bin2.set(this.try_parse_semiatomic());

        Some(())
    }

    pub fn parse_binary_relaxed(&mut self, left: ExprInd, min_priority: Priority) {
        let this = self;
        let (class, pr) = this.get_curr_token_class_and_priority();

        let ind = this.curr_token_ind;
        let is_anything = Rc::new(Cell::new(None));
        let is_anything2 = is_anything.clone();


        if pr.0 >= min_priority {
            if min_priority < pr.1 && class != TokenClass::FlowOp {
                *this <<= move |this|  {this.parse_binary_relaxed(this.expressions_len().prev().unwrap(), min_priority)}
            }
            *this <<= move |this| {
                let prev = this.expressions_len().prev().unwrap();
                match (class, is_anything.get().is_some()) {
                    (TokenClass::None, true) => {
                        *this += ASTExpr::Binary(left, ind, prev);
                    }
                    (TokenClass::None, false) => {
                        *this += ASTExpr::Empty;
                        *this += ASTExpr::Binary(left, ind, prev);
                    }
                    (TokenClass::FlowKw, true) => {
                        if let Some(prev) = this.is_prev_flow_br() {
                            *this.prev_expr_mut().unwrap() = ASTExpr::Flow(left, ind, None, prev.clone());
                        } else if let Some((mhs, rhs)) = this.is_prev_flow_op() {
                            *this.prev_expr_mut().unwrap() = ASTExpr::Flow(left, ind, mhs, rhs.clone());
                        } else {
                            *this += ASTExpr::Flow(left, ind, None, [prev].into());
                        }
                    }
                    (TokenClass::FlowKw, false) => {
                        *this += ASTExpr::Flow(left, ind, None, ExprVec::default());
                    }
                    (TokenClass::FlowOp, true) => {
                        if let Some(prev) = this.is_prev_flow_br() {
                            *this += ASTExpr::UnFlow(ind, Some(left), prev.clone());
                        } else {
                            *this += ASTExpr::UnFlow(ind, Some(left), [prev].into());
                        }
                    }
                    (TokenClass::FlowOp, false) => {
                        *this += ASTExpr::UnFlow(ind, Some(left), ExprVec::default());
                    }
                    (TokenClass::Comma, true) => {
                        if let Some(prev) = this.try_get_prev_tuple() {
                            prev.push_front(left)
                        } else {
                            *this += ASTExpr::Tuple(Rc::new([left, prev].into()));
                        }
                    }
                    (TokenClass::Comma, false) => {
                        *this += ASTExpr::Tuple(Rc::new([left].into()));
                    }
                    (TokenClass::Stick, true) => {
                        if let Some(prev) = this.try_get_prev_either() {
                            prev.push_front(left)
                        } else {
                            *this += ASTExpr::Either(Rc::new([left, prev].into()));
                        }
                    }
                    (TokenClass::Stick, false) => {
                        if let Some(_prev) = this.try_get_prev_either() {
                            // do nothing
                        } else {
                            *this += ASTExpr::Either(Rc::new([left].into()));
                        }
                    }
                };
            };
            this.advance();
            is_anything2.set(this.try_parse_any(pr.1, matches!(class, TokenClass::FlowOp | TokenClass::FlowKw), matches!(class, TokenClass::Comma) ));
        };
    }

    pub fn parse_binary(&mut self, left: ExprInd, min_priority: Priority) {
        let this = self;

        *this <<= move |this| {this.parse_binary_relaxed(this.expressions_len().prev().unwrap(), min_priority)};
        *this <<= move |this| {this.parse_apply_or_stop(this.expressions_len().prev().unwrap(),)};
        this.parse_path_or_stop(left)
    }

    pub fn parse_unary(&mut self, ind: TokenInd, min_priority: Priority) {
        let this = self;
        let (class, (_, pr_right)) = this.get_curr_token_class_and_priority();

        let is_anything = Rc::new(Cell::new(None));
        let is_anything2 = is_anything.clone();

        if min_priority < pr_right && class != TokenClass::FlowOp {
            *this <<= move |this| {this.parse_binary_relaxed(this.expressions_len().prev().unwrap(), min_priority)}
        }
        *this <<= move |this| {
            let expr_len = this.expressions_len();

            match (class, is_anything.get().is_some()) {
                (TokenClass::None, true) => {
                    *this += ASTExpr::Unary(ind, expr_len.prev().unwrap())
                }
                (TokenClass::None, false) => {
                    *this += ASTExpr::Simple(ind,)
                }
                (TokenClass::FlowKw | TokenClass::FlowOp, true) => {
                    if let Some(prev) = this.is_prev_flow_br() {
                        *this.prev_expr_mut().unwrap() = ASTExpr::UnFlow(ind, None, prev.clone());
                    } else if let Some((mhs, rhs)) = this.is_prev_flow_op() {
                        *this.prev_expr_mut().unwrap() = ASTExpr::UnFlow(ind, mhs, rhs.clone());
                    } else {
                        *this += ASTExpr::UnFlow(ind, None, [expr_len.prev().unwrap()].into());
                    }
                }
                (TokenClass::FlowKw | TokenClass::FlowOp, false) => {
                    *this += ASTExpr::UnFlow(ind, None, ExprVec::default());
                }
                (TokenClass::Comma, true) => {
                    *this += ASTExpr::Unary(ind, expr_len.prev().unwrap());
                    *this /= UnexpectedComma;
                }
                (TokenClass::Comma, false) => {
                    *this += ASTExpr::Simple(ind,);
                    *this /= UnexpectedComma;
                }
                (TokenClass::Stick, true) => {
                    if let Some(_prev) = this.try_get_prev_either() {
                        //do nothing
                    } else {
                        *this += ASTExpr::Either(Rc::new(vec![expr_len.prev().unwrap()].into()) )
                    }
                }
                (TokenClass::Stick, false) => {
                    *this += ASTExpr::Simple(ind,)
                }
            };
        };
        this.advance();
        is_anything2.set(this.try_parse_any(pr_right, matches!(class, TokenClass::FlowKw | TokenClass::FlowOp), false));
    }
}
