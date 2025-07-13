use crate::lexer::lex::skip_while_alnum;
use crate::lexer::{State, StringErrorFlags, StringValue, Token, TokenIterator, TokenType};

pub fn lex_string(it: &mut TokenIterator, start: State) -> Token {
    let Some(post_start @ State { char: '"', .. }) = it.current else {
        unreachable!()
    };
    if start.num > post_start.num {
        unreachable!()
    };

    let prefix_len = post_start.num - start.num;
    let start_row = it.row;
    let lines = it.lines.clone();
    let start_line = &lines[start_row];

    let mut quote_count = 1usize;
    it.next_char();

    while let Some(State {
        char: '"',
        num: 1..,
        ..
    }) = it.current
    {
        quote_count += 1;
        it.next_char();
    }

    if quote_count == 2 {
        let suffix_start = post_start.num + 2;
        skip_while_alnum(it);

        let end_pos = if let Some(State { num: num @ 1.., .. }) = it.current {
            num
        } else {
            start_line.len()
        };

        return Token {
            slice: start_line.slice(start.num..end_pos),
            row: start_row,
            col: start.col,
            num: start.num,
            ty: TokenType::String {
                value: StringValue::Empty,
                suffix_len: end_pos - suffix_start,
                prefix_len,
                errors: StringErrorFlags::empty(),
                quote_count: 1,
            },
        };
    }

    let mut prev_char = '\0';
    let mut curr_quote_len = 0;

    let mut line_not_empty = false;

    while let Some(State { char, num: 1.., .. }) = it.current {
        if char == '"' && prev_char != '\\' {
            curr_quote_len += 1;
        } else if let ' ' | '\t' = char {
            curr_quote_len = 0;
        } else {
            line_not_empty = true;
            curr_quote_len = 0;
        }

        prev_char = char;
        it.next_char();

        if curr_quote_len != quote_count {
            continue;
        }

        let pre_suffix = it.current;
        skip_while_alnum(it);

        let (suffix_len, last_pos) = match (pre_suffix, it.current) {
            (
                Some(State { num, .. }),
                Some(State {
                    num: curr_num @ 1..,
                    ..
                }),
            ) => (curr_num - num, curr_num),
            (Some(State { num: num @ 1.., .. }), _) => (start_line.len() - num, start_line.len()),
            (_, _) => (0, start_line.len()),
        };

        return Token {
            slice: start_line.slice(start.num..last_pos),
            row: start_row,
            col: start.col,
            num: start.num,
            ty: TokenType::String {
                value: StringValue::SingleLine(
                    start_line.slice(start.num + quote_count..last_pos - quote_count),
                ),
                suffix_len,
                prefix_len,
                errors: StringErrorFlags::empty(),
                quote_count,
            },
        };
    }

    if line_not_empty {
        return Token {
            slice: start_line.slice(start.num..),
            row: start_row,
            col: start.col,
            num: start.num,
            ty: TokenType::String {
                value: StringValue::SingleLine(start_line.slice(start.num + quote_count..)),
                suffix_len: 0,
                prefix_len,
                errors: StringErrorFlags::UNCLOSED,
                quote_count,
            },
        };
    }

    while let Some(_) = it.current {
        while let Some(State {
            char: ' ' | '\t', ..
        }) = it.current
        {
            it.next_char();
        }

        let mut curr_quote_len = 0;
        let mut last_line = 0;
        while let Some(State { char: '"', num, .. }) = it.current {
            if curr_quote_len == quote_count {
                break;
            }
            if num == 0 {
                curr_quote_len = 0
            }
            curr_quote_len += 1;
            last_line = it.row;
            it.next_char();
        }

        if curr_quote_len != quote_count {
            it.next_line();
            continue;
        }

        let pre_suffix = it.current;
        skip_while_alnum(it);

        let (suffix_len, last_line_len) = match (pre_suffix, it.current) {
            (
                Some(State { num, .. }),
                Some(State {
                    num: curr_num @ 1..,
                    ..
                }),
            ) => (curr_num - num, curr_num),
            (Some(State { num: num @ 1.., .. }), _) => {
                (it.lines[last_line].len() - num, it.lines[last_line].len())
            }
            (_, _) => (0, it.lines[last_line].len()),
        };

        return Token {
            slice: start_line.slice(start.num..),
            row: start_row,
            col: start.col,
            num: start.num,
            ty: TokenType::String {
                value: StringValue::MultiLine {
                    lines: start_row + 1..last_line,
                    last_line_slice: it.lines[last_line].slice(..last_line_len),
                },
                suffix_len,
                prefix_len,
                errors: StringErrorFlags::empty(),
                quote_count,
            },
        };
    }

    Token {
        slice: start_line.slice(start.num..),
        row: start_row,
        col: start.col,
        num: start.num,
        ty: TokenType::String {
            value: StringValue::MultiLine {
                lines: start_row + 1..it.lines.len(),
                last_line_slice: Default::default(),
            },
            suffix_len: 0,
            prefix_len,
            errors: StringErrorFlags::UNCLOSED,
            quote_count,
        },
    }
}
