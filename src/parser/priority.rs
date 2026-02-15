use crate::lexer::{Token, TokenType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenClass {
    None,
    FlowKw,
    FlowOp,
    Comma,
    Stick,
}



#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Close, // ) ] ; …

    Or,  // or else … left
    And, // …<- …<-| : := ::= not and then match while for … right

    Pipe, // >… =… <… |… &… :… left

    Tuple, // , right

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

    // to/do : change
    if it.starts_with(['|', '&', '<', '>', '=', ':']) {
        return (Priority::Pipe, Priority::Tuple);
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
        "and" | "then" | "for" | "when" | "while" | "type" | "match" => (And, And),
        "or" | "else" => (Or, And),
        "return" | "yield" | "not" => (Apply, And),
        "in" | "is" => (Pipe, Range),
        "as" => (Apply, Apply),
        "pub" | "priv" | "mut" | "const" => (Path, Path),
        _ => (Apply, Path),
    }
}

impl Token {
    pub fn priority(&self) -> (Priority, Priority) {
        use Priority::*;
        match self.ty {
            TokenType::Open(_, _) => (Apply, Or),
            TokenType::Close(_) => (Close, Path),
            TokenType::Operator => priority_of_operator(&self.slice),
            TokenType::Comma => (Tuple, Tuple),
            TokenType::Semicolon => (Close, Or),
            TokenType::Colon => (And, And),
            TokenType::ColonEq => (Or, And),
            TokenType::ColonColonEq => (Or, And),
            TokenType::ColonColon => (Path, Path), // left though
            TokenType::Keyword(ref slice) => priority_of_keyword(&slice),
            TokenType::Symbol(_, _) => (Apply, Path),
            TokenType::Character { .. } => (Apply, Path),
            TokenType::String { .. } => (Apply, Path),
            TokenType::Number { .. } => (Apply, Path),
            TokenType::Indent(_) => unreachable!(),
            TokenType::Error => (Apply, Path),
            TokenType::Lifetime => (Path, Path),
            TokenType::OrStick => (Priority::Pipe, Priority::Pipe)
        }
    }

    pub fn class(&self) -> TokenClass {
        match self.ty {
            TokenType::Keyword(ref x) if matches!(x.as_str(),
                "and" | "then" | "for" | "when" | "while" | "type" | "or" | "else" | "match"
            ) => TokenClass::FlowKw,
            TokenType::Colon | TokenType::ColonColonEq | TokenType::ColonEq => TokenClass::FlowOp,
            TokenType::Comma => TokenClass::Comma,
            TokenType::OrStick => TokenClass::Stick,
            _ => TokenClass::None
        }
    }

    pub fn is_flow_kw(&self) -> bool {
        self.class() == TokenClass::FlowKw
    }
    pub fn is_flow_op(&self) -> bool {
        self.class() == TokenClass::FlowOp
    }
}