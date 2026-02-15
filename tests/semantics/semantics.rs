use rolang::lexer::TokenIterator;
#[allow(unused)]
use rolang::parser::visualize::visualize_ast;
use rolang::semantics::{AnalyzeErr, AnalyzedAST};

fn parse(str: &str) -> AnalyzedAST {
    AnalyzedAST::new(TokenIterator::new(str).normalize().parse())
}

#[test]
fn check_semantics() {
    let ast = parse(
        r#"
    f1 a b := a + b
    f2 a b := 
        diff := a - b
        diff * f1 a b 
    "#,
    );

    //dbg!(&ast);

    //eprintln!("{}", visualize_ast(&ast));

    for errs in &ast.errors {
        assert!(matches!(errs, AnalyzeErr::None));
    }
}
#[test]
fn check_semantics_repeating() {
    let ast = parse(
        r#"
    i i := (i := (i := i + i; i := i + i; i) + i; i) + i
    "#,
    );

    //dbg!(&ast);

    //eprintln!("{}", visualize_ast(&ast));

    for errs in &ast.errors {
        assert!(matches!(errs, AnalyzeErr::None));
    }
}
