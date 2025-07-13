use crate::lexer::TokenType::{Close, Ellipsis, Operator};
use crate::lexer::lex::skip_while_op;
use crate::lexer::{State, Token, TokenIterator};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

pub fn lex_operator(it: &mut TokenIterator) -> Token {
    let Some(State { num, col, .. }) = it.current else {
        unreachable!()
    };
    let row = it.row;
    let line = it.lines[row].clone();

    skip_while_op(it);

    let mut is_close = false;

    if matches!((|| Some((it.current?, it.current?.char.general_category())))(),
        Some((
            State{num : 1.., char, ..},
            GeneralCategory::ClosePunctuation | GeneralCategory::FinalPunctuation,
        ))
        if char != ')'
    ) {
        it.next_char();
        is_close = true;
    };

    let slice = if let Some(State {
        num: end_pos @ 1.., ..
    }) = it.current
    {
        line.slice(num..end_pos)
    } else {
        line.slice(num..)
    };

    let ty = if slice == "..." {
        Ellipsis
    } else if is_close {
        Close
    } else {
        Operator
    };

    Token {
        slice,
        row,
        col,
        num,
        ty,
    }
}
