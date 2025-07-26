use crate::lexer::{Token, TokenType};
use crate::parser::{ASTErr, ASTExpr, ExprInd};
use crate::parser::parser_state::ParserState;
use crate::push_fn;

impl<'a, IterT: Iterator<Item = Token>> ParserState<'a, IterT> {
    pub fn parse_path(&mut self, left: ExprInd) {
        let this = self;
        
        let Some(Token{ty: TokenType::ColonColon,..}) = this.current() else { panic!() };
        
        this.parse_path_step(left, vec![]);
    }

    pub fn try_parse_atomic(&mut self) -> Option<()> {
        let this = self;
        use TokenType::*;
        if matches!(this.current().unwrap().ty, 
            | Open(_)
            | Symbol(_) 
            | Argument(_)
            | Number{..}
            | String{..}
            | Character{..}
        ) || matches!(this.current().unwrap().ty, 
            | Keyword(ref n) if n == "_"
        ) {
            push_fn! {this =>
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
        this.try_parse_atomic().unwrap_or_else(|| push_fn!{this =>
            *this /= ASTErr::NotAtomic;
            *this += ASTExpr::Empty;
        })
    }
    
    fn parse_path_step(&mut self, left: ExprInd, mut frags: Vec<ExprInd>) {
        let this = self;
        
        if let TokenType::ColonColon = this.current().unwrap().ty {
            this.advance();
            push_fn!{this => 
                frags.push(this.expressions_len().prev().unwrap());
                this.parse_path_step(left, frags);
            }
            this.parse_atomic();
        } else {
            *this += ASTExpr::Path(left, frags);
        }
    }
}