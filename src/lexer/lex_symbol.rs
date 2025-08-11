use crate::lexer::TokenType::{Error, Keyword, Symbol};
use crate::lexer::lex_char::lex_char;
use crate::lexer::lex_string::lex_string;
use crate::lexer::{State, Token, TokenIterator, is_keyword, skip_while_alnum, SymbolType};

pub fn lex_symbol(it: &mut TokenIterator) -> Token {
    let Some(start @ State { num, col, char: _ }) = it.current else {
        unreachable!()
    };
    let row = it.row;
    let line = it.lines[row].clone();

    macro_rules! slice_end {
        () => {{
            if let Some(State {
                num: curr_num @ 1..,
                ..
            }) = it.current
            {
                curr_num
            } else {
                line.len()
            }
        }};
    }

    it.next_char();
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

            let slice_end: usize = slice_end!();

            let slice = line.slice(num..slice_end);
            let identifier = line.slice(sharp_num + 1..slice_end);
            let prefix = &line[num..sharp_num];
            Token {
                ty: match prefix {
                    "k" => Keyword(identifier),
                    "r" => Symbol(identifier, SymbolType::Default),
                    _ => Error,
                },
                slice,
                row,
                col,
                num,
            }
        }
        _ => {
            let slice = line.slice(num..slice_end!());
            Token {
                ty: if is_keyword(slice.as_str()) {
                    Keyword(slice.clone())
                } else {
                    Symbol(slice.clone(), SymbolType::Default)
                },
                slice,
                row,
                col,
                num,
            }
        }
    }
}
