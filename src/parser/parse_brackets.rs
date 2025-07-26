use crate::lexer::{BracketType, Token, TokenType};
use crate::parser::parser_state::ParserState;
use crate::parser::priority::Priority;
use crate::parser::{ASTErr, ASTExpr, ExprInd, TokenInd};
use crate::push_fn;
use std::cell::Cell;
use std::rc::Rc;

impl<'a, IterT: Iterator<Item = Token>> ParserState<'a, IterT> {
    pub(crate) fn parse_brackets_or_panic(&mut self) {
        self.try_parse_brackets().unwrap()
    }
    pub(crate) fn try_parse_brackets(&mut self) -> Option<()> {
        let this = self;

        let Some(Token {
            ty: TokenType::Open(br_ty),
            ..
        }) = this.current()
        else {
            None?
        };
        let br_ty = *br_ty;

        let open_tok = this.curr_token_ind;

        this.advance();

        this.parse_br_member(br_ty, vec![], open_tok);
        Some(())
    }

    fn parse_br_member(&mut self, br_ty: BracketType, mut exprs: Vec<ExprInd>, op_ind: TokenInd) {
        let this = self;

        let isnt_empty = Rc::new(Cell::new(None));
        let isnt_empty2 = isnt_empty.clone();

        push_fn! { this =>
            let exprs_len = this.expressions_len();
            let cl_ind = this.curr_token_ind;
            // if were any expression, add it
            if isnt_empty.get().is_some() {
                exprs.push(exprs_len.prev().unwrap());
            };
            // ( _<eof>_
            let Some(sep_tok) = this.current() else {
                *this /= ASTErr::Unclosed;
                *this += ASTExpr::Braced(op_ind, exprs, cl_ind);
                // end
                return;
            };
            // ( _;_
            let TokenType::Close(br_ty_cl) = sep_tok.ty else {
                this.advance();
                this.parse_br_member(br_ty, exprs, op_ind);
                return;
            };
            // ( _]_
            if br_ty != br_ty_cl {
                // ( _<dedent>_
                if br_ty_cl == BracketType::Indent {
                    *this /= ASTErr::Unclosed;
                    *this += ASTExpr::Braced(op_ind, exprs, cl_ind);
                    // end
                    return;
                }
                this.advance();
                *this /= ASTErr::Unopened;
                *this += ASTExpr::Braced(op_ind, exprs, cl_ind);
                // parse as there were no members yet
                this.parse_br_member(br_ty, vec![], op_ind);
                this.parse_binary(exprs_len, Priority::Comma);
                return;
            };
            // ( _)_
            this.advance();
            *this += ASTExpr::Braced(op_ind, exprs, cl_ind);
            // end
        }
        push_fn! {isnt_empty2 = this.try_parse_any(Priority::Comma)}
    }
}
