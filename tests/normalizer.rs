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
        a := 1 (        ## (
            b := 2
                c := 3  ## ;
            d := 3      ## )
        )               ## ;
        e := 4          ## ;)
    ",
    );

    let opens = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Open))
        .count();
    let closes = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Close))
        .count();
    let semis = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Semicolon))
        .count();

    dbg!(tokens);

    assert_eq!([opens, closes, semis], [3, 3, 3]);
}

#[test]
fn no_semicolon_after_control_flow() {
    let tokens = norm_tokens(
        "
        x then
            x
        else
            x
        x
    ",
    );
    /* <(>
        x then <(>
            x <)>
        else <(>
            x <)> <;>
        x <;> <)>
    */

    let flows = tokens
        .windows(2)
        .filter(|w| matches!(w[0], TokenType::Keyword(_)) && matches!(w[1], TokenType::Semicolon))
        .count();
    let opens = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Open))
        .count();
    let closes = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Close))
        .count();
    let semis = tokens
        .iter()
        .filter(|t| matches!(t, TokenType::Semicolon))
        .count();

    dbg!(tokens);

    assert_eq!(
        flows, 0,
        "Semicolon must not follow control flow keyword directly"
    );
    assert_eq!([opens, closes, semis], [3, 3, 2]);
}
