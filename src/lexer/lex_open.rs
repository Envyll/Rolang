use crate::lexer::TokenType::Open;
use crate::lexer::{BracketType, State, SymbolType, Token, TokenIterator};

pub fn lex_open(it: &mut TokenIterator, State { col, num, .. }: State, ty: SymbolType) -> Token {
    let char = it.current.unwrap().char;
    let row = it.row;
    it.next_char();
    let with_stick = if let Some(State {
        num: 1..,
        char: '|',
        ..
    }) = it.current
    {
        it.next_char();
        true
    } else {
        false
    };

    let ty = Open(
        match char {
            '(' => BracketType::Round { with_stick },
            '[' => BracketType::Square { with_stick },
            '{' => BracketType::Curly { with_stick },
            _ => BracketType::NonAscii { with_stick },
        },
        ty,
    );

    let slice = it.lines[row].slice(num..num + char.len_utf8() + with_stick as usize);

    Token {
        slice,
        row,
        col,
        num,
        ty,
    }
}
