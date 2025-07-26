use rolang::lexer::*;

fn lex(src: &str) -> Vec<TokenType> {
    let res = TokenIterator::new(src)
        .map(|tok| tok.ty)
        .skip_while(|tok| matches!(tok, TokenType::Indent(_)))
        .collect();
    dbg!(&res);
    res
}

fn lex_one(src: &str) -> TokenType {
    lex(src)[0].clone()
}

#[test]
fn simple_char() {
    let TokenType::Character { value, errors, .. } = lex_one("'a'") else {
        panic!()
    };
    assert_eq!(value, 'a');
    assert!(errors.is_empty());
}

#[test]
fn empty_char() {
    let TokenType::Character { errors, .. } = lex_one("''") else {
        panic!()
    };
    assert!(errors.contains(CharErrorFlags::EMPTY));
}

#[test]
fn escaped_newline() {
    let TokenType::Character { value, errors, .. } = lex_one(r"'\n'") else {
        panic!()
    };
    assert_eq!(value, '\n');
    assert!(errors.is_empty());
}

#[test]
fn bad_escape() {
    let TokenType::Character { value, errors, .. } = lex_one(r"'\z'") else {
        panic!()
    };
    assert_eq!(value, '\0');
    assert!(errors.contains(CharErrorFlags::BAD_ESC_SEQUENCE));
}

#[test]
fn hex_escape() {
    let TokenType::Character { value, errors, .. } = lex_one(r"'\x41'") else {
        panic!()
    };
    assert_eq!(value, 'A');
    assert!(errors.is_empty());
}

#[test]
fn unicode_escape_u() {
    let TokenType::Character { value, errors, .. } = lex_one(r"'\u0041'") else {
        panic!()
    };
    assert_eq!(value, 'A');
    assert!(errors.is_empty());
}

#[test]
#[allow(non_snake_case)]
fn unicode_escape_U() {
    let TokenType::Character { value, errors, .. } = lex_one(r"'\U00000041'") else {
        panic!()
    };
    assert_eq!(value, 'A');
    assert!(errors.is_empty());
}

#[test]
fn unterminated_char() {
    let TokenType::Character { errors, .. } = lex_one("a'a") else {
        panic!()
    };
    assert!(errors.contains(CharErrorFlags::UNCLOSED));
}

#[test]
fn argument() {
    let TokenType::Argument(value) = lex_one("'a") else {
        panic!()
    };
    assert_eq!(value, "a");
}
