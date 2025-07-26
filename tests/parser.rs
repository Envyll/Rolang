use rolang::lexer::TokenIterator;
use rolang::parser::AST;
use rolang::parser::visualize::visualize_ast;

fn parse(str: &str) -> AST {
    TokenIterator::new(str).normalize().parse()
}

// fi-xme: write tests
// (but me doesn't want…………………………… ◑﹏◐ )

#[test]
fn visualize() {
    // もうそのままでOKと思う。=￣ω￣= (silly)
    println!("{}", visualize_ast(&parse("
    abc [ 1) + 3 ]
    abc.def::zzz
    then xyz
    else q: uvw
    + a * b
    * a + b
    a #\"
    abc
    \" c
    
    (a => b) type [a] :=
        self.fun d := 
        self.xxx := a
    ")));
}