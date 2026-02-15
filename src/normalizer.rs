use crate::ImStr;
use crate::lexer::{BracketType, Lines, SymbolType, Token, TokenIterator, TokenType};
use crate::parser::priority::Priority;
use std::mem;

#[derive(Debug, Clone)]
pub struct NormalizedTokenIterator {
    iter: TokenIterator,
    curr: Option<Token>,
    next: Option<Token>,
    stack: Vec<usize>,
    good_for_semicolon: bool,
    good_for_indent: bool,
}

impl NormalizedTokenIterator {
    pub fn new(mut iter: TokenIterator) -> Self {
        Self {
            curr: iter.next(),
            next: iter.next(),
            iter,
            stack: vec![],
            good_for_semicolon: false,
            good_for_indent: true,
        }
    }
}

impl TokenIterator {
    pub fn normalize(self) -> NormalizedTokenIterator {
        NormalizedTokenIterator::new(self)
    }
}

impl Lines for NormalizedTokenIterator {
    fn lines(&self) -> &[ImStr] {
        self.iter.lines()
    }
}

impl Iterator for NormalizedTokenIterator {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        'all: loop {
            let peak = self.stack.last();
            break 'all match &self.curr {
                Some(Token {
                    ty: TokenType::Indent(len),
                    slice,
                    row,
                    ..
                }) if self.good_for_indent
                    && (peak.is_none() || *len > *peak.unwrap_or(&0))
                    && is_prev_good_for_indent_or_dedent(&self.next) =>
                {
                    self.good_for_semicolon = false;
                    self.good_for_indent = false;
                    let row = *row;
                    let slice = slice.clone();
                    self.stack.push(*len);
                    self.curr = mem::replace(&mut self.next, self.iter.next());
                    Some(Token {
                        slice,
                        row,
                        col: 0,
                        num: 0,
                        ty: TokenType::Open(BracketType::Indent, SymbolType::Default),
                    })
                }
                Some(Token {
                    ty: TokenType::Indent(len),
                    slice,
                    row,
                    ..
                }) if *len < *peak.unwrap_or(&0)
                    && is_prev_good_for_indent_or_dedent(&self.next) =>
                {
                    self.good_for_semicolon = true;
                    self.good_for_indent = false;
                    let row = *row;
                    let slice = slice.clone();
                    self.stack.pop();
                    Some(Token {
                        slice,
                        row,
                        col: 0,
                        num: 0,
                        ty: TokenType::Close(BracketType::Indent),
                    })
                }
                Some(Token {
                    ty: TokenType::Indent(len),
                    slice,
                    row,
                    ..
                }) if self.good_for_semicolon
                    && *len == *peak.unwrap_or(&0)
                    && is_prev_good_for_semicolon(&self.next) =>
                {
                    self.good_for_semicolon = false;
                    self.good_for_indent = false;
                    let row = *row;
                    let slice = slice.clone();
                    self.curr = mem::replace(&mut self.next, self.iter.next());
                    Some(Token {
                        slice,
                        row,
                        col: 0,
                        num: 0,
                        ty: TokenType::Semicolon,
                    })
                }
                Some(Token {
                    ty: TokenType::Indent(_),
                    ..
                }) => {
                    self.curr = mem::replace(&mut self.next, self.iter.next());
                    continue 'all;
                }
                Some(t) => {
                    self.good_for_indent = is_next_good_for_indent(t);
                    self.good_for_semicolon = is_next_good_for_semicolon(t);
                    mem::replace(
                        &mut self.curr,
                        mem::replace(&mut self.next, self.iter.next()),
                    )
                }
                None if self.good_for_semicolon => {
                    self.good_for_semicolon = false;
                    Some(Token {
                        slice: Default::default(),
                        row: self.iter.lines().len(),
                        col: 0,
                        num: 0,
                        ty: TokenType::Semicolon,
                    })
                }
                None if !self.stack.is_empty() => {
                    self.stack.pop();
                    Some(Token {
                        slice: Default::default(),
                        row: self.iter.lines().len(),
                        col: 0,
                        num: 0,
                        ty: TokenType::Close(BracketType::Indent),
                    })
                }
                None => None,
            };
        }
    }
}

fn is_prev_good_for_indent_or_dedent(tok: &Option<Token>) -> bool {
    match tok {
        Some(Token {
            ty: TokenType::Indent(_),
            ..
        }) => false,
        _ => true,
    }
}

fn is_next_good_for_semicolon(tok @ Token { ty, .. }: &Token) -> bool {
    match ty {
        TokenType::Close(_)
        | TokenType::Symbol(..)
        | TokenType::Number { .. }
        | TokenType::String { .. }
        | TokenType::Character { .. }
        | TokenType::Colon
        | TokenType::ColonEq
        | TokenType::ColonColonEq => true,
        TokenType::Keyword { .. } if !tok.is_flow_kw() => true,
        _ => false,
    }
}

fn is_prev_good_for_semicolon(tok: &Option<Token>) -> bool {
    let Some(tok @ Token { ty, .. }) = tok else {
        return false;
    };
    match ty {
        TokenType::Semicolon | TokenType::Comma | TokenType::Indent(_) => false,
        _ if tok.priority().0 <= Priority::And => false,
        _ => true,
    }
}

fn is_next_good_for_indent(tok @ Token { ty, .. }: &Token) -> bool {
    match ty {
        TokenType::Open(_, _) | TokenType::Colon | TokenType::ColonEq | TokenType::ColonColonEq => {
            true
        }
        TokenType::Keyword(_) if tok.is_flow_kw() => true,
        _ => false,
    }
}
