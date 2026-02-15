use super::Result;
use crate::generation::simple_int_functions::generate;
use inkwell::context::Context;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;

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


#[test]
fn test_match_tuple() -> Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;


    let target = Target::from_triple(&TargetMachine::get_default_triple())?;
    let machine = target.create_target_machine(
        &TargetMachine::get_default_triple(),
        TargetMachine::get_host_cpu_name().to_str()?,
        TargetMachine::get_host_cpu_features().to_str()?,
        OptimizationLevel::Default,
        RelocMode::Default,
        CodeModel::JITDefault,
    ).unwrap();

    let context = Context::create();
    let module = context.create_module("");

    module.set_triple(&TargetMachine::get_default_triple());

    generate("
    f x y := x, y match
        0, _ : 0
        _, 0 : 1
        1, 1 | 2, (2|1) : 2
        _ : 3
    ", &module)?;

    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?;

    module.set_data_layout(&machine.get_target_data().get_data_layout());
    module.run_passes("default<Os>", &machine, PassBuilderOptions::create())?;
    module.print_to_stderr();

    let f = unsafe{engine.get_function::<unsafe extern "C" fn (u64, u64) -> u64>("f")?};

    assert_eq!(unsafe{f.call(0, 0)}, 0);
    assert_eq!(unsafe{f.call(1, 0)}, 1);
    assert_eq!(unsafe{f.call(0, 1)}, 0);
    assert_eq!(unsafe{f.call(1, 1)}, 2);
    assert_eq!(unsafe{f.call(2, 2)}, 2);
    assert_eq!(unsafe{f.call(2, 1)}, 2);
    assert_eq!(unsafe{f.call(1, 2)}, 3);
    assert_eq!(unsafe{f.call(3, 3)}, 3);

    Ok(())
}


#[test]
#[ignore] //planned
fn test_match_bindings() -> Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;

    let context = Context::create();
    let module = context.create_module("");

    module.set_triple(&TargetMachine::get_default_triple());

    generate("
    f x y := x, y match
        0, x : x
        x, 0 : x
        a as _, _ : 0 a + 1 a - 1
    ", &module)?;

    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?;

    module.print_to_stderr();

    let f = unsafe{engine.get_function::<unsafe extern "C" fn (u64, u64) -> u64>("f")?};

    assert_eq!(unsafe{f.call(0, 0)}, 0);
    assert_eq!(unsafe{f.call(1, 0)}, 1);
    assert_eq!(unsafe{f.call(0, 2)}, 2);
    assert_eq!(unsafe{f.call(2, 3)}, 4);

    Ok(())
}

#[test]
#[ignore] //planned
fn test_let_match() -> Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;

    let context = Context::create();
    let module = context.create_module("");

    module.set_triple(&TargetMachine::get_default_triple());

    generate("
    f x y :=
        0, n | n, _ := x, y
        n
    ", &module)?;

    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?;

    module.print_to_stderr();

    let f = unsafe{engine.get_function::<unsafe extern "C" fn (u64, u64) -> u64>("f")?};

    assert_eq!(unsafe{f.call(0, 0)}, 0);
    assert_eq!(unsafe{f.call(1, 0)}, 1);
    assert_eq!(unsafe{f.call(0, 2)}, 2);
    assert_eq!(unsafe{f.call(2, 3)}, 3);

    Ok(())
}


#[test]
#[ignore] //planned
fn test_fn_match() -> Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;

    let context = Context::create();
    let module = context.create_module("");

    module.set_triple(&TargetMachine::get_default_triple());

    generate("
    f x y :=
    f 0 0 ::= 42
    f n 0 ::= n * 2
    f 0 m ::= m * 3
    f n m ::= n + m
    ", &module)?;

    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?;

    module.print_to_stderr();

    let f = unsafe{engine.get_function::<unsafe extern "C" fn (u64, u64) -> u64>("f")?};

    assert_eq!(unsafe{f.call(0, 0)}, 42);
    assert_eq!(unsafe{f.call(1, 0)}, 2);
    assert_eq!(unsafe{f.call(0, 2)}, 3);
    assert_eq!(unsafe{f.call(2, 3)}, 5);

    Ok(())
}

