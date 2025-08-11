use rolang::lexer::{TokenIterator, TokenType};

#[test]
fn test_indent_normalization() {
    let src = "(\nlet x := 3\n    y := 4\nz := 5\n)";
    let tokens: Vec<_> = TokenIterator::new(src).normalize().collect();

    let mut semicolon_cnt = 0;

    for token in &tokens {
        if let TokenType::Semicolon = token.ty {
            semicolon_cnt += 1
        };
        println!("{:?} {}", token.ty, token.row);
    }
    assert_eq!(semicolon_cnt, 2);
}

fn norm_tokens(str: &str) -> Vec<TokenType> {
    TokenIterator::new(str).normalize().map(|t| t.ty).collect()
}

#[test]
fn simple_indent_block() {
    let tokens = norm_tokens(
        "           ## (
        a := 1 (|       ## (
            b := 2      ##
                c := 3  ## ;
            d := 3      ## )
        |)              ## ;
        e := 4          ## ;)
    ",
    );

    let opens = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Open(_, _)))
        .count();
    let closes = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Close(_)))
        .count();
    let semis = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Semicolon))
        .count();

    dbg!(tokens);

    assert_eq!([opens, closes, semis], [3, 3, 3]);
}

#[test]
fn control_flow() {
    let str = "##(
        \tx then:   ##(
        \t\tx       ##)
        \telse:     ##(
        \t\tx       ##);
        \tx         ##;)
    ";

    dbg!(TokenIterator::new(str).map(|t| t.ty).collect::<Vec<_>>());

    let tokens = norm_tokens(str);

    let opens = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Open(_, _)))
        .count();
    let closes = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Close(_)))
        .count();
    let semis = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Semicolon))
        .count();

    dbg!(tokens);

    assert_eq!([opens, closes, semis], [3, 3, 2]);
}
