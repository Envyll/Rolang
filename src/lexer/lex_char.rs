use crate::lexer::lex::{is_alnum, skip_while_alnum};
use crate::lexer::{CharErrorFlags, State, Token, TokenIterator, TokenType};

pub fn lex_char(it: &mut TokenIterator, start: State) -> Token {
    let row = it.row;
    let col = start.col;
    let num = start.num;
    let line = it.lines[row].clone();

    let Some(State {
        num: quo_num,
        char: '\'',
        ..
    }) = it.current
    else {
        unreachable!()
    };

    let prefix_len = quo_num - num;

    it.next_char();

    let Some(State {
        num: fst_num @ 1..,
        char: fst_char,
        ..
    }) = it.current
    else {
        return Token {
            slice: line.slice(num..),
            row,
            col,
            num,
            ty: TokenType::Character {
                value: '\0',
                prefix_len,
                suffix_len: 0,
                errors: CharErrorFlags::UNCLOSED,
            },
        };
    };

    let is_fst_alnum = is_alnum(fst_char);
    skip_while_alnum(it);

    match (fst_char, is_fst_alnum, it.current) {
        // …'\…'…
        ('\\', ..) => {
            it.next_char();
            skip_while_alnum(it);

            if let Some(State {
                num: suff_num_m1 @ 1..,
                char: '\'',
                ..
            }) = it.current
            {
                it.next_char();
                skip_while_alnum(it);

                let slice_end = if let Some(State {
                    num: slice_end @ 1..,
                    ..
                }) = it.current
                {
                    slice_end
                } else {
                    line.len()
                };

                let (value, flags) = parse_esc(&line[fst_num + 1..suff_num_m1]);

                Token {
                    slice: line.slice(num..slice_end),
                    row,
                    col,
                    num,
                    ty: TokenType::Character {
                        value,
                        prefix_len,
                        suffix_len: slice_end - suff_num_m1 - 1,
                        errors: flags,
                    },
                }
            // …'\…
            } else {
                let slice_end = if let Some(State {
                    num: slice_end @ 1..,
                    ..
                }) = it.current
                {
                    slice_end
                } else {
                    line.len()
                };

                Token {
                    slice: line.slice(num..slice_end),
                    row,
                    col,
                    num,
                    ty: TokenType::Character {
                        value: '\'',
                        prefix_len,
                        suffix_len: 0,
                        errors: CharErrorFlags::UNCLOSED,
                    },
                }
            }
        }
        // …''…
        ('\'', ..) => {
            it.next_char();

            // …'''…
            let ok = matches!(
                it.current,
                Some(State {
                    num: 1..,
                    char: '\'',
                    ..
                })
            );
            if ok {
                it.next_char();
            };

            skip_while_alnum(it);

            let slice_end = if let Some(State {
                num: slice_end @ 1..,
                ..
            }) = it.current
            {
                slice_end
            } else {
                line.len()
            };

            Token {
                slice: line.slice(num..slice_end),
                row,
                col,
                num,
                ty: TokenType::Character {
                    value: '\'',
                    prefix_len,
                    suffix_len: slice_end - fst_num - 1 - ok as usize,
                    errors: if ok {
                        CharErrorFlags::empty()
                    } else {
                        CharErrorFlags::EMPTY
                    },
                },
            }
        }
        // …'_'…
        (
            _,
            true,
            Some(State {
                char: '\'',
                num: suff_num_m1 @ 1..,
                ..
            }),
        ) if suff_num_m1 == fst_num + fst_char.len_utf8() => {
            it.next_char();
            skip_while_alnum(it);

            let slice_end = if let Some(State {
                num: slice_end @ 1..,
                ..
            }) = it.current
            {
                slice_end
            } else {
                line.len()
            };

            Token {
                slice: line.slice(num..slice_end),
                row,
                col,
                num,
                ty: TokenType::Character {
                    value: fst_char,
                    prefix_len,
                    suffix_len: slice_end - suff_num_m1 - 1,
                    errors: CharErrorFlags::empty(),
                },
            }
        }
        // …'…
        (_, true, _) => {
            let slice_end = if let Some(State {
                num: slice_end @ 1..,
                ..
            }) = it.current
            {
                slice_end
            } else {
                line.len()
            };

            Token {
                slice: line.slice(num..slice_end),
                row,
                col,
                num,
                ty: if prefix_len == 0 {
                    TokenType::Argument(line.slice(fst_num..slice_end))
                } else {
                    TokenType::Character {
                        value: fst_char,
                        prefix_len,
                        suffix_len: slice_end - fst_num,
                        errors: CharErrorFlags::UNCLOSED,
                    }
                },
            }
        }
        _ => {
            it.next_char();

            // …'.'…
            if let Some(State {
                num: suff_num_m1 @ 1..,
                char: '\'',
                ..
            }) = it.current
            {
                it.next_char();
                skip_while_alnum(it);

                let slice_end = if let Some(State {
                    num: slice_end @ 1..,
                    ..
                }) = it.current
                {
                    slice_end
                } else {
                    line.len()
                };

                Token {
                    slice: line.slice(num..slice_end),
                    row,
                    col,
                    num,
                    ty: TokenType::Character {
                        value: fst_char,
                        prefix_len,
                        suffix_len: slice_end - suff_num_m1 - 1,
                        errors: CharErrorFlags::empty(),
                    },
                }
            // …'.
            } else {
                Token {
                    slice: line.slice(num..fst_num + 1),
                    row,
                    col,
                    num,
                    ty: TokenType::Character {
                        value: '\'',
                        prefix_len,
                        suffix_len: 0,
                        errors: CharErrorFlags::UNCLOSED,
                    },
                }
            }
        }
    }
}

fn parse_esc(seq: &str) -> (char, CharErrorFlags) {
    let Some(fst_esc_ch) = seq.chars().next() else {
        return ('\'', CharErrorFlags::empty());
    };
    match (fst_esc_ch, seq.len()) {
        ('0'..='9', _) => {
            let Ok(res) = u32::from_str_radix(seq, 10) else {
                return ('\0', CharErrorFlags::BAD_ESC_SEQUENCE);
            };
            let Ok(ch) = char::try_from(res) else {
                return ('\0', CharErrorFlags::INVALID_CODEPOINT);
            };
            (ch, CharErrorFlags::empty())
        }
        ('o', 4) => {
            let seq = &seq[1..];
            let Ok(res) = i8::from_str_radix(seq, 8)
            // i8 to filter smth like '\o333'
            else {
                return ('\0', CharErrorFlags::BAD_ESC_SEQUENCE);
            };
            (char::from(res as u8), CharErrorFlags::empty())
        }
        ('x', 3) => {
            let seq = &seq[1..];
            let Ok(res) = i8::from_str_radix(seq, 16) else {
                return ('\0', CharErrorFlags::BAD_ESC_SEQUENCE);
            };
            (char::from(res as u8), CharErrorFlags::empty())
        }
        ('u', 5) => {
            let seq = &seq[1..];
            let Ok(res) = u32::from_str_radix(seq, 16) else {
                return ('\0', CharErrorFlags::BAD_ESC_SEQUENCE);
            };
            let Ok(ch) = char::try_from(res) else {
                return ('\0', CharErrorFlags::INVALID_CODEPOINT);
            };
            (ch, CharErrorFlags::empty())
        }
        ('U', 9) => {
            let seq = &seq[1..];
            let Ok(res) = u32::from_str_radix(seq, 16) else {
                return ('\0', CharErrorFlags::BAD_ESC_SEQUENCE);
            };
            let Ok(ch) = char::try_from(res) else {
                return ('\0', CharErrorFlags::INVALID_CODEPOINT);
            };
            (ch, CharErrorFlags::empty())
        }
        ('\\', 1) => ('\\', CharErrorFlags::empty()),
        ('a', 1) => ('\x07', CharErrorFlags::empty()),
        ('b', 1) => ('\x08', CharErrorFlags::empty()),
        ('e', 1) => ('\x1B', CharErrorFlags::empty()),
        ('n', 1) => ('\n', CharErrorFlags::empty()),
        ('r', 1) => ('\r', CharErrorFlags::empty()),
        ('t', 1) => ('\t', CharErrorFlags::empty()),
        _ => ('\0', CharErrorFlags::BAD_ESC_SEQUENCE),
    }
}
