use crate::lexer::Token;
use crate::parser::parser_state::ParserState;
use crate::parser::priority::Priority;
use crate::parser::{ASTErr, ASTExpr, ExprInd, ExprVec, TokenInd};
use crate::push_fn;
use std::cell::Cell;
use std::rc::Rc;

impl<'a, IterT: Iterator<Item = Token>> ParserState<'a, IterT> {
    pub fn try_parse_any(&mut self, min_priority: Priority, was_flow: bool) -> Option<()> {
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

        push_fn! {this => 
            if is_bin.get().is_some() {
                if !was_flow || this.is_prev_flow_br().is_none() {
                    this.parse_binary(this.expressions_len().prev().unwrap(), min_priority);
                }
            } else {
                this.parse_unary(ind, min_priority);
            };
        }
        is_bin2.set(this.try_parse_semiatomic());

        Some(())
    }

    pub fn parse_binary_relaxed(&mut self, left: ExprInd, min_priority: Priority) {
        let this = self;
        let curr = this.current().unwrap();
        let pr = curr.priority();
        let is_flow_kw = curr.is_flow_kw();
        let is_flow_op = curr.is_flow_op();
        let ind = this.curr_token_ind;
        let is_anything = Rc::new(Cell::new(None));
        let is_anything2 = is_anything.clone();

        if pr.0 >= min_priority {
            if min_priority < pr.1 && !is_flow_op {
                push_fn! {this.parse_binary_relaxed(this.expressions_len().prev().unwrap(), min_priority)}
            }
            push_fn! {this => 
                let prev = this.expressions_len().prev().unwrap();
                if is_flow_op {
                    if is_anything.get().is_none() {
                        *this += ASTExpr::UnFlow(ind, Some(left), ExprVec::default());
                    } else if let Some(prev) = this.is_prev_flow_br() {
                        *this += ASTExpr::UnFlow(ind, Some(left), prev.clone());
                    } else {
                        *this += ASTExpr::UnFlow(ind, Some(left), [prev].into());   
                        if let Some(_) = this.is_prev_flow_op() {
                            *this /= ASTErr::UnexpectedFlowOp;
                        }
                    }  
                } else if is_flow_kw {
                    if is_anything.get().is_none() {
                        *this += ASTExpr::Flow(left, ind, None, ExprVec::default());
                    } else if let Some(prev) = this.is_prev_flow_br() {
                        *this.prev_expr_mut().unwrap() = ASTExpr::Flow(left, ind, None, prev.clone());
                    } else if let Some((mhs, rhs)) = this.is_prev_flow_op() {
                        *this.prev_expr_mut().unwrap() = ASTExpr::Flow(left, ind, mhs, rhs.clone());
                    } else {
                        *this += ASTExpr::Flow(left, ind, None, [prev].into());    
                    }  
                } else if is_anything.get().is_some() {
                    *this += ASTExpr::Binary(left, ind, prev);
                } else {
                    *this += ASTExpr::Empty;
                    *this += ASTExpr::Binary(left, ind, this.expressions_len().prev().unwrap());
                };
            }
            this.advance();
            is_anything2.set(this.try_parse_any(pr.1, is_flow_kw || is_flow_op));
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
        let curr = this.current().unwrap();
        let pr_right = curr.priority().1;
        let is_flow_kw = curr.is_flow_kw();
        let is_flow_op = curr.is_flow_op();
        let is_flow = is_flow_op || is_flow_kw;

        let is_anything = Rc::new(Cell::new(None));
        let is_anything2 = is_anything.clone();        
        
        if min_priority < pr_right && !is_flow_op {
            push_fn! {this.parse_binary_relaxed(this.expressions_len().prev().unwrap(), min_priority)}
        }
        push_fn! {this =>
            let expr_len = this.expressions_len();
            if is_flow {
                if is_anything.get().is_none() {
                    *this += ASTExpr::UnFlow(ind, None, ExprVec::default());
                } else if let Some(prev) = this.is_prev_flow_br() {
                    *this.prev_expr_mut().unwrap() = ASTExpr::UnFlow(ind, None, prev.clone());
                } else if let Some((mhs, rhs)) = this.is_prev_flow_op() {
                    *this.prev_expr_mut().unwrap() = ASTExpr::UnFlow(ind, mhs, rhs.clone());
                    if is_flow_op {
                        *this /= ASTErr::UnexpectedFlowOp;
                    }
                } else {
                    *this += ASTExpr::UnFlow(ind, None, [expr_len.prev().unwrap()].into());    
                }                
            } else if is_anything.get().is_some() {
                    *this += ASTExpr::Unary(ind, expr_len.prev().unwrap())
            } else {
                *this += ASTExpr::Simple(ind,)
            };
        }
        this.advance();
        is_anything2.set(this.try_parse_any(pr_right, is_flow));
    }
}
