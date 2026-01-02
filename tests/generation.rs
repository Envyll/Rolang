use inkwell::builder::BuilderError;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::OptimizationLevel;
use inkwell::targets::{InitializationConfig, Target};
use rolang::lexer::TokenIterator;
use rolang::semantics::AnalyzedAST;
use rolang::Result;

fn generate(str: &str, module: &Module) -> Result<(), BuilderError> {
    AnalyzedAST::new(TokenIterator::new(str).normalize().parse()).build_module(module)
}

#[test]
fn test_generation() -> Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;

    let context = Context::create();
    let module = context.create_module("");

    generate("

    f a b := a + b

    g a b :=
        diff := a - b
        diff * f b a

    h x y z w :=
        sum :=
            mul := x * y
            mul + z
        diff := w - sum
        diff 

    ", &module)?;
    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?;

    let f = unsafe{engine.get_function::<unsafe extern "C" fn (u64, u64) -> u64>("f") }?;

    assert_eq!(dbg!(unsafe {f.call(0, 1)}), 1);
    assert_eq!(dbg!(unsafe {f.call(3, 5)}), 8);


    let g = unsafe{engine.get_function::<unsafe extern "C" fn (u64, u64) -> u64>("g") }?;

    assert_eq!(dbg!(unsafe {g.call(3, 3)}), 0);
    assert_eq!(dbg!(unsafe {g.call(5, 3)}), 16);

    let h = unsafe{engine.get_function::<unsafe extern "C" fn (u64, u64, u64, u64) -> u64>("h") }?;

    assert_eq!(dbg!(unsafe {h.call(4, 9, 5, 42)}), 1);

    Ok(())
}
