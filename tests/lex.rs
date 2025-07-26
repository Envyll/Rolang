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
fn test_numbers_simple() {
    let tokens = lex("42 0xFF 0xff 0b1010 3.14 3. 1e10 1.2e-3");
    for (n, token) in tokens.iter().enumerate() {
        match token {
            TokenType::Number { errors, .. } => {
                let errors = errors.bits();
                const NO_START_DIGITS_ERR: u8 = NumberErrorFlags::NO_START_DIGITS.bits();
                const FREE_DOT_ERR: u8 = NumberErrorFlags::FREE_DOT.bits();
                match (n, errors) {
                    (2, NO_START_DIGITS_ERR) => (),
                    (5, FREE_DOT_ERR) => (),
                    (_, 0) => {}
                    _ => panic!("Unexpected number error"),
                }
            }
            _ => panic!("Expected number token"),
        }
    }
    assert_eq!(tokens.len(), 8);
}

#[test]
fn test_strings_simple() {
    let tokens = lex("\"abc\" \"\" \"\\\"escaped\\\"\"");
    assert_eq!(tokens.len(), 3);
    for token in tokens {
        match token {
            TokenType::String { .. } => {}
            _ => panic!("Expected string"),
        }
    }
}

#[test]
fn test_full_expression() {
    let src = "mut x := fact 3; println \"Result: {}\" + x";
    let tokens = lex(src);

    let expected = [
        "Keyword",
        "Symbol",
        "ColonEq",
        "Symbol",
        "Number",
        "Semicolon",
        "Symbol",
        "String",
        "Operator",
        "Symbol",
    ];

    assert_eq!(tokens.len(), expected.len());

    for (token, expected_str) in tokens.iter().zip(expected.iter()) {
        match (token, *expected_str) {
            (TokenType::Keyword { .. }, "Keyword") => {}
            (TokenType::Symbol { .. }, "Symbol") => {}
            (TokenType::Operator, "Operator") => {}
            (TokenType::ColonEq, "ColonEq") => {}
            (TokenType::Number { .. }, "Number") => {}
            (TokenType::String { .. }, "String") => {}
            (TokenType::Semicolon { .. }, "Semicolon") => {}
            _ => panic!("Unexpected token: {token:?}, expected {expected_str}"),
        }
    }
}

#[test]
fn test_identifiers() {
    assert!(matches!(lex_one("foo"), TokenType::Symbol { .. }));
    assert!(
        matches!(lex_one("then"), TokenType::Keyword( identifier ) if identifier.as_str()=="then")
    );
    assert!(
        matches!(lex_one("k#if"), TokenType::Keyword( identifier ) if identifier.as_str()=="if")
    );
    assert!(matches!(lex_one("_x1"), TokenType::Symbol { .. }));
}

#[test]
fn test_numbers_2() {
    assert!(matches!(
        lex_one("123"),
        TokenType::Number {
            dot_pos: None,
            exp_pos: None,
            ..
        }
    ));
    assert!(matches!(lex_one("0x1F"), TokenType::Number { .. }));
    assert!(matches!(
        lex_one("3.14i32"),
        TokenType::Number {
            dot_pos: Some(_),
            suf_pos: Some(_),
            ..
        }
    ));
    assert!(matches!(
        lex_one("1e+10u64"),
        TokenType::Number {
            exp_pos: Some(_),
            suf_pos: Some(_),
            ..
        }
    ));
    assert!(matches!(lex_one("0b1010"), TokenType::Number { .. }));

    let t = lex_one("0xG");
    assert!(
        matches!(t, TokenType::Number{ errors, .. } if errors.contains(NumberErrorFlags::BAD_SUFFIX))
    )
}

#[test]
fn test_operators() {
    assert!(matches!(lex_one("+"), TokenType::Operator));
    assert!(matches!(lex_one("++--"), TokenType::Operator { .. }));
}

#[test]
fn test_characters() {
    assert!(
        matches!(lex_one("'a'"), TokenType::Character{ value: 'a', errors, .. } if errors.is_empty())
    );
    assert!(matches!(
        lex_one("'\\n'"),
        TokenType::Character { value: '\n', .. }
    ));
    assert!(
        matches!(lex_one("'\\o999'"), TokenType::Character{ errors: e, .. } if e.contains(CharErrorFlags::BAD_ESC_SEQUENCE))
    );
}

#[test]
fn test_strings_2() {
    assert!(
        matches!(lex_one("\"hello\""), TokenType::String{ value: StringValue::SingleLine(s), .. } if s.as_str()=="hello")
    );
    assert!(
        matches!(lex_one("\"\"\"bad multi\nline\"\"\""), TokenType::String{ value: StringValue::SingleLine{..}, errors, .. } if errors.contains(StringErrorFlags::UNCLOSED))
    );
    assert!(matches!(
        lex_one("\"\"\"\nmulti\nline\"\"\""),
        TokenType::String {
            value: StringValue::MultiLine { .. },
            ..
        }
    ));
}

#[test]
fn test_braces() {
    use TokenType::*;
    assert_eq!(lex("").as_slice(), []);
    assert_eq!(
        lex("[]"),
        [
            Open(BracketType::Square { with_stick: false }),
            Close(BracketType::Square { with_stick: false })
        ]
    );
    assert_eq!(
        lex("[| |]"),
        [
            Open(BracketType::Square { with_stick: true }),
            Close(BracketType::Square { with_stick: true })
        ]
    );
    assert_eq!(
        lex("(||)"),
        [
            Open(BracketType::Round { with_stick: true }),
            Close(BracketType::Round { with_stick: true })
        ]
    );
    assert_eq!(
        lex("｟〗"),
        [
            Open(BracketType::NonAscii { with_stick: false }),
            Close(BracketType::NonAscii { with_stick: false })
        ]
    );
    assert_eq!(
        lex("[== ==]"),
        [
            Open(BracketType::Square { with_stick: false }),
            Operator,
            Operator,
            Close(BracketType::Square { with_stick: false })
        ]
    );
    assert_eq!(
        lex("⁅|abc|⁆"),
        [
            Open(BracketType::NonAscii { with_stick: true }),
            Symbol("abc".into()),
            Close(BracketType::NonAscii { with_stick: true })
        ]
    );
    assert_eq!(
        lex("【|||】"),
        [
            Open(BracketType::NonAscii { with_stick: true }),
            Operator,
            Close(BracketType::NonAscii { with_stick: true })
        ]
    );
    assert_eq!(lex("{|}").len(), 2);
    assert_eq!(
        lex("||»|«||"),
        [
            Operator,
            Close(BracketType::NonAscii { with_stick: true }),
            Operator,
            Open(BracketType::NonAscii { with_stick: true }),
            Operator,
        ]
    );
    assert_eq!(
        lex("©｟日"),
        [
            Operator,
            Open(BracketType::NonAscii { with_stick: false }),
            Symbol("日".into())
        ]
    );
}
