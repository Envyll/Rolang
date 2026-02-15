use crate::lexer::{Token, TokenType};
use crate::parser::parser_state::ParserState;
use crate::parser::{ASTErr, ASTExpr, ExprInd};

impl<'a, IterT: Iterator<Item = Token>> ParserState<'a, IterT> {
    pub fn parse_path(&mut self, left: ExprInd) {
        let this = self;

        let Some(Token {
            ty: TokenType::ColonColon,
            ..
        }) = this.current()
        else {
            panic!()
        };

        this.parse_path_step(left, vec![]);
    }

    pub fn try_parse_atomic(&mut self) -> Option<()> {
        let this = self;
        use TokenType::*;
        let ty = &this.current().unwrap().ty;
        if matches!(
            ty,
            Open(_, _) | Symbol(..) | Number { .. } | String { .. } | Character { .. }
        ) || matches!(ty,
            | Keyword(n) if n == "_"
        ) {
            *this <<= move |this| {
                if this.try_parse_brackets().is_none() {
                    let ind = this.curr_token_ind;
                    this.advance();
                    *this += ASTExpr::Simple(ind);
                };
            }
        } else {
            None?
        };
        Some(())
    }

    pub fn parse_atomic(&mut self) {
        let this = self;
        this.try_parse_atomic().unwrap_or_else(|| {
            *this <<= move |this| {
                *this += ASTExpr::Empty;
                *this /= ASTErr::NotAtomic;
            }
        })
    }

    fn parse_path_step(&mut self, left: ExprInd, mut frags: Vec<ExprInd>) {
        let this = self;

        if let TokenType::ColonColon = this.current().unwrap().ty {
            this.advance();
            *this <<= move |this| {
                frags.push(this.expressions_len().prev().unwrap());
                this.parse_path_step(left, frags);
            };
            this.parse_atomic();
        } else {
            *this += ASTExpr::Path(left, frags.into());
        }
    }
}
