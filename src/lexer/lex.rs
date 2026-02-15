use crate::ImStr;
use crate::lexer::TokenType::*;
use crate::lexer::lex_char::lex_char;
use crate::lexer::lex_number::lex_number;
use crate::lexer::lex_open::lex_open;
use crate::lexer::lex_operator::lex_operator;
use crate::lexer::lex_string::lex_string;
use crate::lexer::lex_symbol::lex_symbol;
use crate::lexer::{Lines, State, SymbolType, Token, TokenIterator};
use unicode_properties::*;

impl TokenIterator {
    pub fn new(str: &str) -> Self {
        let mut buffer = std::string::String::with_capacity(str.len() + 2);
        let mut ranges = vec![0usize, 1usize];

        buffer.push(' ');

        for line in str.lines() {
            buffer += " ";
            buffer += line;
            ranges.push(buffer.len());
        }

        let buffer = ImStr::from(buffer);

        let lines = ranges
            .iter()
            .zip(ranges.iter().skip(1))
            .map(|(r1, r2)| buffer.slice(r1..r2))
            .collect::<Vec<_>>();

        TokenIterator {
            row: 0,
            iter: lines[0].char_indices().enumerate(),
            lines,
            current: Some(State {
                num: 0,
                col: 0,
                char: ' ',
            }),
            prev_is_dot: false,
        }
    }

    pub(super) fn next_char(&mut self) -> &mut Self {
        self.prev_is_dot = false;
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
        self.prev_is_dot = false;
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
}

impl Lines for TokenIterator {
    #[inline]
    fn lines(&self) -> &[ImStr] {
        &self.lines
    }
}

impl Iterator for TokenIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        Some('all: loop {
            let start @ State { num, col, char } = self.current?;
            let row = self.row;

            macro_rules! slice_end {
                () => {
                    self.current
                        .map(|s| s.num)
                        .take_if(|s| *s > 0)
                        .unwrap_or(self.lines[row].len())
                };
            }

            if num == 0 {
                return lex_indent(self);
            }

            break 'all match (char, char.general_category()) {
                (' ' | '\t', _) => {
                    skip_whitespaces_and_get_len(self);
                    continue 'all;
                }
                ('"', ..) => lex_string(self, start),
                ('\'', ..) => lex_char(self, start),

                ('#' | '@' | '$', ..) => {
                    self.next_char();

                    let s_ty = match char {
                        '#' if self.current.is_some_and(|c| c.char == '#') => {
                            self.next_line();
                            continue 'all;
                        }
                        '#' => SymbolType::Attribute,
                        '@' => SymbolType::Pointer,
                        '$' => SymbolType::Borrow,
                        _ => unreachable!(),
                    };

                    if let Some(
                        GeneralCategory::OpenPunctuation | GeneralCategory::InitialPunctuation,
                    ) = (|| Some(self.current?.char.general_category()))()
                    {
                        return Some(lex_open(self, start, s_ty));
                    };
                    skip_while_alnum(self);
                    let slice = self.lines[row].slice(num..slice_end!());
                    Token {
                        ty: Symbol(slice.clone(), s_ty),
                        slice,
                        row,
                        col,
                        num,
                    }
                }
                ('`', ..) => {
                    self.next_char();
                    skip_while_alnum(self);
                    let slice = self.lines[row].slice(num..slice_end!());
                    Token {
                        ty: Lifetime,
                        slice,
                        row,
                        col,
                        num,
                    }
                }
                ('0'..='9', ..) if !self.prev_is_dot => lex_number(self),
                (',', ..) => {
                    self.next_char();
                    Token {
                        slice: ','.into(),
                        row,
                        col,
                        num,
                        ty: Comma,
                    }
                }
                (';', ..) => {
                    self.next_char();
                    Token {
                        slice: ';'.into(),
                        row,
                        col,
                        num,
                        ty: Semicolon,
                    }
                }
                // ( [ { …
                (_, GeneralCategory::OpenPunctuation | GeneralCategory::InitialPunctuation) => {
                    lex_open(self, start, SymbolType::Default)
                }
                // ) ] } …
                (_, GeneralCategory::ClosePunctuation | GeneralCategory::FinalPunctuation) => {
                    lex_operator(self)
                }
                _ if is_op(char) => lex_operator(self),
                _ if is_alnum(char) => lex_symbol(self),
                _ => {
                    self.next_char();
                    Token {
                        slice: char.into(),
                        row,
                        col,
                        num,
                        ty: Error,
                    }
                }
            };
        })
    }
}

fn lex_indent(it: &mut TokenIterator) -> Option<Token> {
    it.next_char();
    let len = skip_whitespaces_and_get_len(it);
    let Some(State { num, .. }) = it.current else {
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
    !matches!(char, |';'| ',' | '#' | '@' | '$' | '`' | '\'' | '"' | '_')
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
    skip_while(is_op, it)
}

pub(super) fn skip_whitespaces_and_get_len(it: &mut TokenIterator) -> usize {
    let mut len = 0;
    while let Some(State { char, num: 1.., .. }) = it.current {
        match char {
            ' ' => len += 1,
            '\t' => len = len + 4 - len % 4,
            _ => return len,
        };
        it.next_char();
    }
    len
}
