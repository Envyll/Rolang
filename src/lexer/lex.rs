use crate::ImStr;
use crate::lexer::TokenType::*;
use crate::lexer::lex_char::lex_char;
use crate::lexer::lex_number::lex_number;
use crate::lexer::lex_operator::lex_operator;
use crate::lexer::lex_string::lex_string;
use crate::lexer::lex_symbol::lex_symbol;
use crate::lexer::{State, Token, TokenIterator};
use unicode_properties::*;

impl TokenIterator {
    pub fn new(mut str: &str) -> Self {
        if str.is_empty() {
            str = " ";
        }

        let lines: Vec<_> = str
            .lines()
            .map(|str| ImStr::from(" ".to_string() + str.trim_end()))
            .collect();

        let mut result = TokenIterator {
            row: 0,
            iter: lines[0].char_indices().enumerate(),
            lines,
            current: None,
        };
        result.next_char();
        result
    }

    pub(super) fn next_char(&mut self) -> &mut Self {
        loop {
            if let Some((col, (num, char))) = self.iter.next() {
                self.current = Some(State { num, col, char });
                break;
            }
            if self.row + 1 == self.lines.len() {
                self.current = None;
                break;
            }
            self.row += 1;
            self.iter = self.lines[self.row].char_indices().enumerate();
        }
        self
    }

    pub(super) fn next_line(&mut self) -> &mut Self {
        loop {
            if self.row + 1 == self.lines.len() {
                self.current = None;
                self.iter = ImStr::default().char_indices().enumerate();
                break;
            }
            self.row += 1;
            self.iter = self.lines[self.row].char_indices().enumerate();
            if let Some((_, (_, char))) = self.iter.next() {
                self.current = Some(State {
                    num: 0,
                    col: 0,
                    char,
                });
                break;
            }
        }
        self
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn lines(&self) -> &Vec<ImStr> {
        &self.lines
    }
}

impl Iterator for TokenIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        'all: loop {
            let start @ State { num, col, char } = self.current?;
            let row = self.row;

            if num == 0 {
                return lex_indent(self);
            }

            break 'all match (char, char.general_category(), char.general_category_group()) {
                (' ' | '\t', _, _) => {
                    skip_whitespaces_and_get_len(self);
                    continue 'all;
                }
                ('_', ..) | (_, _, GeneralCategoryGroup::Letter) => Some(lex_symbol(self)),
                ('"', ..) => Some(lex_string(self, start)),
                ('\'', ..) => Some(lex_char(self, start)),
                ('#', ..) => {
                    self.next_char();
                    if let Some(State { char: '#', .. }) = self.current {
                        self.next_line();
                        continue 'all;
                    } else {
                        Some(Token {
                            slice: '#'.into(),
                            row,
                            col,
                            num,
                            ty: Sharp,
                        })
                    }
                }
                ('0'..='9', ..) => Some(lex_number(self)),
                (',', ..) => {
                    self.next_char();
                    Some(Token {
                        slice: ','.into(),
                        row,
                        col,
                        num,
                        ty: Comma,
                    })
                }
                (';', ..) => {
                    self.next_char();
                    Some(Token {
                        slice: ';'.into(),
                        row,
                        col,
                        num,
                        ty: Semicolon,
                    })
                }
                (':', ..) => {
                    self.next_char();
                    if let Some(State {
                        char: ':',
                        num: 1..,
                        ..
                    }) = self.current
                    {
                        self.next_char();
                        if let Some(State {
                            char: '=',
                            num: 1..,
                            ..
                        }) = self.current
                        {
                            self.next_char();
                            Some(Token {
                                slice: "::=".into(),
                                row,
                                col,
                                num,
                                ty: ColonColonEq,
                            })
                        } else {
                            Some(Token {
                                slice: "::".into(),
                                row,
                                col,
                                num,
                                ty: ColonColon,
                            })
                        }
                    } else if let Some(State {
                        char: '=',
                        num: 1..,
                        ..
                    }) = self.current
                    {
                        self.next_char();
                        Some(Token {
                            slice: ":=".into(),
                            row,
                            col,
                            num,
                            ty: ColonEq,
                        })
                    } else {
                        Some(Token {
                            slice: ':'.into(),
                            row,
                            col,
                            num,
                            ty: Colon,
                        })
                    }
                }
                ('(', ..) => {
                    self.next_char();
                    Some(Token {
                        slice: char.into(),
                        row,
                        col,
                        num,
                        ty: Open,
                    })
                }
                (')', ..) => {
                    self.next_char();
                    Some(Token {
                        slice: char.into(),
                        row,
                        col,
                        num,
                        ty: Close,
                    })
                }
                // [ { …
                (
                    _,
                    GeneralCategory::OpenPunctuation | GeneralCategory::InitialPunctuation,
                    GeneralCategoryGroup::Punctuation,
                ) => {
                    self.next_char();
                    skip_while_op(self);

                    let ty = if matches!((|| Some((self.current?, self.current?.char.general_category())))(), 
                    Some((
                        State{char, col: col2 @ 1.., ..},
                        GeneralCategory::ClosePunctuation | GeneralCategory::FinalPunctuation,
                    )) if char != ')' && col2 != col + 1)
                    {
                        self.next_char();
                        Operator
                    } else {
                        Open
                    };

                    let slice = if let Some(State {
                        num: curr_num @ 1..,
                        ..
                    }) = self.current
                    {
                        self.lines[row].slice(num..curr_num)
                    } else {
                        self.lines[row].slice(num..)
                    };

                    Some(Token {
                        slice,
                        row,
                        col,
                        num,
                        ty,
                    })
                }
                (_, _, GeneralCategoryGroup::Punctuation | GeneralCategoryGroup::Symbol) => {
                    Some(lex_operator(self))
                }
                _ => {
                    self.next_char();
                    Some(Token {
                        slice: char.into(),
                        row,
                        col,
                        num,
                        ty: Error,
                    })
                }
            };
        }
    }
}

fn lex_indent(it: &mut TokenIterator) -> Option<Token> {
    let len = skip_whitespaces_and_get_len(it);
    let Some(State { num: num @ 1.., .. }) = it.current else {
        return None;
    };
    let row = it.row;
    let slice = it.lines[row].slice(0..num);

    Some(Token {
        ty: Indent(len),
        slice,
        row,
        num: 0,
        col: 0,
    })
}

pub(super) fn skip_while(mut f: impl FnMut(char) -> bool, it: &mut TokenIterator) {
    while let Some(State { char, num: 1.., .. }) = it.current {
        if !f(char) {
            return;
        }
        it.next_char();
    }
}

pub(super) fn is_alnum(char: char) -> bool {
    char == '_'
        || matches!(
            char.general_category_group(),
            GeneralCategoryGroup::Letter
                | GeneralCategoryGroup::Number
                | GeneralCategoryGroup::Mark
        )
}

#[inline]
pub(super) fn skip_while_alnum(it: &mut TokenIterator) {
    skip_while(is_alnum, it)
}

pub(super) fn is_op(char: char) -> bool {
    !matches!(char, |';'| ',' | '#' | '\'' | '"' | '_' | ':')
        && matches!(
            char.general_category_group(),
            GeneralCategoryGroup::Punctuation | GeneralCategoryGroup::Symbol
        )
        && !matches!(
            char.general_category(),
            |GeneralCategory::OpenPunctuation| GeneralCategory::InitialPunctuation
                | GeneralCategory::ClosePunctuation
                | GeneralCategory::FinalPunctuation
        )
}

#[inline]
pub(super) fn skip_while_op(it: &mut TokenIterator) {
    let mut len = 0usize;
    let mut dots = 0usize;

    skip_while(
        move |ch| {
            len += 1;
            if ch == '.' {
                dots += 1;
            }
            (len != 3 || len != dots) && is_op(ch)
        },
        it,
    )
}

pub(super) fn skip_whitespaces_and_get_len(it: &mut TokenIterator) -> usize {
    let mut len = 0;
    while let Some(State { char, num, .. }) = it.current {
        if num == 0 {
            len = 0
        }
        len += match char {
            ' ' => 1,
            '\n' => 4,
            _ => return len,
        };
        it.next_char();
    }
    len
}
