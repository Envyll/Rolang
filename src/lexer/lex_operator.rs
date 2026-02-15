use crate::lexer::TokenType::{Close, Colon, ColonColon, ColonColonEq, ColonEq, Operator, OrStick};
use crate::lexer::lex::skip_while_op;
use crate::lexer::{BracketType, State, Token, TokenIterator};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

pub fn lex_operator(it: &mut TokenIterator) -> Token {
    let Some(State { num, col, .. }) = it.current else {
        unreachable!()
    };
    let row = it.row;
    let line = it.lines[row].clone();

    let mut iter_clone = it.iter.clone();
    skip_while_op(it);

    if let (
        Some(GeneralCategory::ClosePunctuation | GeneralCategory::FinalPunctuation),
        Some(State {
            num: curr_num @ 1..,
            col: curr_col,
            char,
        }),
    ) = (
        (|| Some(it.current?.char.general_category()))(),
        &mut it.current,
    ) {
        // lone close bracket
        if *curr_col == col {
            let char = *char;
            it.next_char();
            return Token {
                slice: line.slice(num..num + char.len_utf8()),
                row,
                col,
                num,
                ty: Close(match char {
                    ')' => BracketType::Round { with_stick: false },
                    ']' => BracketType::Square { with_stick: false },
                    '}' => BracketType::Curly { with_stick: false },
                    _ => BracketType::NonAscii { with_stick: false },
                }),
            };
        }
        // close bracket with stick
        if *curr_col == col + 1 && line.as_bytes()[num] == b'|' {
            let char = *char;
            it.next_char();
            return Token {
                slice: line.slice(num..num + 1 + char.len_utf8()),
                row,
                col,
                num,
                ty: Close(match char {
                    ')' => BracketType::Round { with_stick: true },
                    ']' => BracketType::Square { with_stick: true },
                    '}' => BracketType::Curly { with_stick: true },
                    _ => BracketType::NonAscii { with_stick: true },
                }),
            };
        }
        // go back one char when stick |||'] -> ||'|]
        if line.as_bytes()[*curr_num - 1] == b'|' {
            *curr_num -= 1;
            *curr_col -= 1;
            *char = '|';
            for _ in col..*curr_col {
                iter_clone.next();
            }
            it.iter = iter_clone;
        }
    }

    let slice = if let Some(State {
        num: end_pos @ 1.., ..
    }) = it.current
    {
        line.slice(num..end_pos)
    } else {
        line.slice(num..)
    };

    let ty = match &slice as &str {
        "|" => OrStick,
        ":" => Colon,
        "::" => ColonColon,
        ":=" => ColonEq,
        "::=" => ColonColonEq,
        "." | "?." => {
            it.prev_is_dot = true;
            Operator
        }
        _ => Operator,
    };

    Token {
        slice,
        row,
        col,
        num,
        ty,
    }
}
