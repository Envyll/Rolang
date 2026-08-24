use inkwell::module::Module;
use rolang::AResult;
use rolang::lexer::TokenIterator;
use rolang::semantics::AnalyzedAST;

pub mod generic;
mod matches;
mod simple_int_functions;
mod tuples;

type Result<T = (), E = Box<dyn std::error::Error>> = rolang::Result<T, E>;

fn generate(str: &str, module: &Module) -> AResult {
    let ast = TokenIterator::new(str).normalize().parse();
    //dbg!(ast);
    //eprintln!("{}", visualize_ast(&ast));
    let res = AnalyzedAST::analyze(ast);

    if !res.errors.is_empty() {
        let first_error = res.errors.iter().next().unwrap();
        anyhow::bail!(
            "At {1}:{2} : {0:} ",
            first_error.1.first().unwrap(),
            res.ast[res.ast.expressions[first_error.0.0].0].row,
            res.ast[res.ast.expressions[first_error.0.0].0].col,
        );
    }

    res.build_module(module)
}
