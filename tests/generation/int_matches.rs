use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::targets::{InitializationConfig, Target};
use crate::generation::simple_int_functions::generate;
use rolang::Result;

#[test]
fn test_match_recursion() -> Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;

    let context = Context::create();
    let module = context.create_module("");

    generate("
    fib_internal n curr prev := n match
        0 : curr
        _ : fib_internal (n - 1) (curr + prev) curr  

    fib n := fib_internal n 0 1

    fact n start :=
        n match {
            0 : start
            _ : fact (n - 1) (start * n)
        }
    ", &module)?;

    module.print_to_stderr();

    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?;
    
    let fib = unsafe{engine.get_function::<unsafe extern "C" fn (u64) -> u64>("fib")?};
    let fact = unsafe{engine.get_function::<unsafe extern "C" fn (u64, u64) -> u64>("fact")?};
    
    assert_eq!(unsafe{fib.call(3)}, 2);
    assert_eq!(unsafe{fib.call(0)}, 0);
    assert_eq!(unsafe{fib.call(5)}, 5);
    
    assert_eq!(unsafe{fact.call(0, 1)}, 1);
    assert_eq!(unsafe{fact.call(3, 1)}, 6);
    
    Ok(())
}

#[test]
fn test_match() -> Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;

    let context = Context::create();
    let module = context.create_module("");

    generate("
    f x := x match
        0 : 1
        1 : 0
        2 | 3 : 3 - x
        _ : x - 4
    ", &module)?;

    module.print_to_stderr();

    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?;

    let f = unsafe{engine.get_function::<unsafe extern "C" fn (u64) -> u64>("f")?};

    assert_eq!(unsafe{f.call(0)}, 1);
    assert_eq!(unsafe{f.call(1)}, 0);
    assert_eq!(unsafe{f.call(2)}, 1);
    assert_eq!(unsafe{f.call(3)}, 0);
    assert_eq!(unsafe{f.call(4)}, 0);
    assert_eq!(unsafe{f.call(5)}, 1);
    assert_eq!(unsafe{f.call(6)}, 2);

    Ok(())
}

