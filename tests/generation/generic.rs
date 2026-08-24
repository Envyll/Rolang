use crate::generation::generate;
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::targets::{InitializationConfig, Target};

#[test]
fn test_generic() -> super::Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;

    let context = Context::create();
    let module = context.create_module("");

    generate(
        "
        f i (x, y) := i match
            0 | 2 : x
            _ : y

        g i x y z w as int := f (i * 2) (f i ((x, y), (z, w)))
    ",
        &module,
    )?;

    module.print_to_stderr();

    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?;

    unsafe {
        let g = engine.get_function::<unsafe extern "C" fn(u64, u64, u64, u64, u64) -> u64>("g")?;
        assert_eq!(g.call(0, 5, 6, 7, 8), 5);
        assert_eq!(g.call(1, 5, 6, 7, 8), 7);
        assert_eq!(g.call(2, 5, 6, 7, 8), 6);
        assert_eq!(g.call(3, 5, 6, 7, 8), 8);
    }

    Ok(())
}
