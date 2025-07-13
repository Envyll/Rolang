use crate::lexer::TokenType::{Error, Keyword, Symbol};
use crate::lexer::lex_char::lex_char;
use crate::lexer::lex_string::lex_string;
use crate::lexer::{State, Token, TokenIterator, is_keyword, skip_while_alnum};

pub fn lex_symbol(it: &mut TokenIterator) -> Token {
    let Some(start @ State { num, col, char: _ }) = it.current else {
        unreachable!()
    };
    let row = it.row;
    let line = it.lines[row].clone();

    let right_bound = |mb_state| {
        if let Some(State {
            num: curr_num @ 1..,
            ..
        }) = mb_state
        {
            curr_num
        } else {
            line.len()
        }
    };

    skip_while_alnum(it);

    match it.current {
        Some(State { char: '"', .. }) => lex_string(it, start),
        Some(State { char: '\'', .. }) => lex_char(it, start),
        Some(State {
            char: '#',
            num: sharp_num,
            ..
        }) => {
            it.next_char();
            skip_while_alnum(it);

            let rbound: usize = right_bound(it.current);

            let slice = line.slice(num..rbound);
            let identifier = line.slice(sharp_num + 1..rbound);
            let prefix = &line[num..sharp_num];
            Token {
                ty: match prefix {
                    "k" => Keyword(identifier),
                    "r" => Symbol(identifier),
                    _ => Error,
                },
                slice,
                row,
                col,
                num,
            }
        }
        _ => {
            let slice = line.slice(num..right_bound(it.current));
            Token {
                ty: if is_keyword(slice.as_str()) {
                    Keyword(slice.clone())
                } else {
                    Symbol(slice.clone())
                },
                slice,
                row,
                col,
                num,
            }
        }
    }
}
