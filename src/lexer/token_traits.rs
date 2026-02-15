use crate::lexer::{Token, TokenType};

impl Token {
    pub fn is_op(&self, str: &str) -> bool {
        let TokenType::Operator = self.ty else {
            return false;
        };
        self.slice == str
    }

    pub fn is_kw(&self, str: &str) -> bool {
        let TokenType::Keyword(ref slice) = self.ty else {
            return false;
        };
        slice == str
    }
}
