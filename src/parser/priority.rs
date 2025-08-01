use crate::lexer::{Token, TokenType};

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Close, // ) ] ; …

    Comma, // ,

    Or,  // or else … left
    And, // …<- …<-| : := ::= not and then match while for … right

    Pipe, // >… =… <… |… &… :… left

    Range, // .. right

    BitOr,    // || right
    BitXor,   // ^^ right
    BitAnd,   // && right
    BitNot,   // ~~ right
    BitShift, // << >> left

    Add, // + - left
    UnAdd,
    Mul,   // * /
    Pow,   // ** right
    Apply, // apply left, anything nullary; => as … right
    Path,  // ::
}

pub fn priority_of_operator(it: &str) -> (Priority, Priority) {
    let it = if it.starts_with('?') { &it[1..] } else { it };

    if let "." = it {
        return (Priority::Path, Priority::Path);
    }

    let it = if it.starts_with('.') { &it[1..] } else { it };

    if it.starts_with("=>") {
        return (Priority::Apply, Priority::Apply);
    }
    if it.starts_with("->") {
        return (Priority::Apply, Priority::Apply);
    }
    if it.starts_with("<<") {
        return (Priority::BitShift, Priority::Add);
    }
    if it.starts_with(">>") {
        return (Priority::BitShift, Priority::Add);
    }
    if it.starts_with("||") {
        return (Priority::BitOr, Priority::BitOr);
    }
    if it.starts_with("&&") {
        return (Priority::BitAnd, Priority::BitAnd);
    }
    if it.starts_with("<-") {
        return (Priority::And, Priority::And);
    }
    if it.starts_with("~~") {
        return (Priority::BitNot, Priority::BitNot);
    }
    if it.starts_with("^^") {
        return (Priority::BitXor, Priority::BitXor);
    }
    if it.starts_with("..") {
        return (Priority::Apply, Priority::Path);
    }
    if it.starts_with('.') {
        return (Priority::Range, Priority::Range);
    }

    if it.starts_with("**") {
        return (Priority::Pow, Priority::Pow);
    }

    if it.starts_with(['|', '&', '<', '>', '=', ':']) {
        return (Priority::Pipe, Priority::Range);
    }

    if it.starts_with(['*', '/', '\\', '%']) {
        return (Priority::Mul, Priority::Pow);
    }
    if it.starts_with(['+', '-']) {
        return (Priority::Add, Priority::UnAdd);
    }

    (Priority::Apply, Priority::Path)
}

pub fn priority_of_keyword(it: &str) -> (Priority, Priority) {
    use Priority::*;
    match it {
        "and" | "then" | "for" | "match" | "while" | "not" => (And, And),
        "when" => (And, Pipe),
        "or" | "else" => (Or, And),
        "return" | "yield" => (Apply, Or),
        "in" | "is" => (Pipe, Range),
        "as" | "type" => (Apply, Apply),
        "pub" | "priv" | "mut" => (Path, Path),
        _ => (Apply, Path),
    }
}

impl Token {
    pub fn priority(&self) -> (Priority, Priority) {
        use Priority::*;
        match self.ty {
            TokenType::Open(_) => (Apply, Comma),
            TokenType::Close(_) => (Close, Path),
            TokenType::Operator => priority_of_operator(&self.slice),
            TokenType::Comma => (Comma, Comma),
            TokenType::Semicolon => (Close, Comma),
            TokenType::Sharp => (Path, Path),
            TokenType::Colon => (And, And),
            TokenType::ColonEq => (And, And),
            TokenType::ColonColon => (Path, Path), // left though
            TokenType::ColonColonEq => (And, And),
            TokenType::Symbol(_) => (Apply, Path),
            TokenType::Keyword(ref slice) => priority_of_keyword(&slice),
            TokenType::Argument(_) => (Apply, Path),
            TokenType::Character { .. } => (Apply, Path),
            TokenType::String { .. } => (Apply, Path),
            TokenType::Number { .. } => (Apply, Path),
            TokenType::Indent(_) => unreachable!(),
            TokenType::Error => (Apply, Path),
        }
    }
}
