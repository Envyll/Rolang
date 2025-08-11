use crate::lexer::TokenType::Number;
use crate::lexer::lex::is_alnum;
use crate::lexer::{NumberErrorFlags, State, Token, TokenIterator, skip_while_alnum};

pub fn lex_number(it: &mut TokenIterator) -> Token {
    
    let Some(State {
        num: start_num,
        col: start_col,
        char: fst_ch,
    }) = it.current
    else {
        unreachable!()
    };
    let row = it.row;
    let line = it.lines[it.row].clone();
    it.next_char();

    let start_pos = match (fst_ch, it.current) {
        (
            '0',
            Some(State {
                char: 'x' | 'o' | 'b',
                num: 1..,
                ..
            }),
        ) => {
            it.next_char();
            start_num + 2
        }
        _ => start_num,
    };

    let mut dot_pos = None;
    let mut exp_pos = None;
    let mut suf_pos = None;
    let mut errors = NumberErrorFlags::empty();

    // <0>.<1>e<2>
    let mut stage = 0;
    let mut has_digit = [Some(fst_ch != '0'), None, None];

    'parse_loop: while let Some(State {
        char,
        num: curr_num @ 1..,
        ..
    }) = it.current
    {
        match char {
            '0'..='9' | 'A'..='F' => {
                has_digit[stage] = Some(true);
                it.next_char();
                continue 'parse_loop;
            }
            '\'' if has_digit[stage] == Some(true) => {
                it.next_char();
                continue 'parse_loop;
            }
            'i' | 'u' | 's' | 'f' | '_' => {
                suf_pos = Some(curr_num);
                skip_while_alnum(it);
                break 'parse_loop;
            }
            'p' | 'e' if stage != 2 => {
                exp_pos = Some(curr_num);
                stage = 2;
                has_digit[stage] = Some(false);

                it.next_char();
                if let Some(State {
                    char: '+' | '-',
                    num: 1..,
                    ..
                }) = it.current
                {
                    it.next_char();
                }
                if let Some(State {
                    char: '\'',
                    num: 1..,
                    ..
                }) = it.current
                {
                    it.next_char();
                    errors |= NumberErrorFlags::BAD_EXPONENT;
                }

                continue 'parse_loop;
            }
            '.' if stage == 0 => {
                dot_pos = Some(curr_num);
                stage = 1;
                has_digit[stage] = Some(false);
                it.next_char();
                continue 'parse_loop;
            }

            _ => {
                break 'parse_loop;
            }
        };
    }

    if let Some(State {
        num: curr_num @ 1..,
        ..
    }) = it.current.filter(|s| is_alnum(s.char))
    {
        suf_pos = Some(curr_num);
        skip_while_alnum(it);
        errors |= NumberErrorFlags::BAD_SUFFIX;
    };

    let end_pose = match it.current {
        Some(State {
            num: curr_num @ 1..,
            ..
        }) => curr_num,
        _ => line.len(),
    };

    const STAGE_ERRORS: [(usize, NumberErrorFlags); 3] = [
        (0, NumberErrorFlags::NO_START_DIGITS),
        (1, NumberErrorFlags::FREE_DOT),
        (2, NumberErrorFlags::BAD_EXPONENT),
    ];

    for (i, flag) in STAGE_ERRORS {
        if has_digit[i] == Some(false) {
            errors |= flag;
        }
    }

    Token {
        slice: line.slice(start_num..end_pose),
        row,
        col: start_col,
        num: start_num,
        ty: Number {
            start_pos,
            dot_pos,
            exp_pos,
            suf_pos,
            errors,
        },
    }
}
