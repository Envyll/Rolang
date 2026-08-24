use crate::generation::generate;
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::targets::{InitializationConfig, Target};

#[test]
fn test_tuples() -> super::Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;

    let context = Context::create();
    let module = context.create_module("");

    generate(
        r#"
        ## f_to x y z w := f (x, (y, z)) w

        f ((x, (y, z)) as (int, (int, int))) (w as int) := w, z, y, x

        g (z as (int, int)) := { w := (1 z, 0 z); w }

        h () := ()

        i ((x,) as (int,)) := x

        min diff z := z match:
            0, _ | _, 0 : diff
            x, y : min (diff + 1) (x - 1, y - 1)

    "#,
        &module,
    )?;

    module.print_to_stderr();

    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?;

    let f = unsafe {
        engine
            .get_function::<unsafe extern "C" fn(u64, u64, u64, u64) -> (u64, u64, u64, u64)>("f")?
    };

    let g = unsafe { engine.get_function::<unsafe extern "C" fn((u64, u64)) -> (u64, u64)>("g")? };

    let h = unsafe { engine.get_function::<unsafe extern "C" fn(()) -> ()>("h")? };

    let i = unsafe { engine.get_function::<unsafe extern "C" fn((i64,)) -> i64>("i")? };

    let min =
        unsafe { engine.get_function::<unsafe extern "C" fn(i64, (i64, i64)) -> i64>("min")? };

    unsafe {
        assert_eq!(f.call(0, 0, 0, 0), (0, 0, 0, 0));
        assert_eq!(f.call(1, 2, 3, 4), (4, 3, 2, 1));

        assert_eq!(g.call((1, 2)), (2, 1));

        assert_eq!(h.call(()), ());
        assert_eq!(i.call((1,)), 1);

        assert_eq!(min.call(0, (1, 2)), 1);
        assert_eq!(min.call(0, (5, 3)), 3);
        assert_eq!(min.call(0, (7, 7)), 7);
        assert_eq!(min.call(0, (1_0000_0000, 1_0000_0000_0000)), 1_0000_0000);
        assert_eq!(min.call(0, (0, 0)), 0);
    }

    Ok(())
}

#[test]
fn test_tuples_creation() -> super::Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;

    let context = Context::create();
    let module = context.create_module("");

    generate(
        r#"
    f (x as int) := ()
    g (x as int) (y as int) := x, y
    h (x as int) := (x,)

    i x := g x x, h x

    j x := 0 (1 (i x))
    "#,
        &module,
    )?;

    module.print_to_stderr();

    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?;

    let f = unsafe { engine.get_function::<unsafe extern "C" fn(u64) -> ()>("f")? };
    let g = unsafe { engine.get_function::<unsafe extern "C" fn(u64, u64) -> (u64, u64)>("g")? };
    let h = unsafe { engine.get_function::<unsafe extern "C" fn(u64) -> (u64,)>("h")? };

    let j = unsafe { engine.get_function::<unsafe extern "C" fn(u64) -> u64>("j")? };

    unsafe {
        assert_eq!(f.call(0), ());
        assert_eq!(g.call(0, 1), (0, 1));
        assert_eq!(g.call(1, 0), (1, 0));
        assert_eq!(h.call(2,), (2,));
        //bad (call conv issue)
        //assert_eq!(i.call(3, ), ((3, 3), (3,) ));
        assert_eq!(j.call(3,), 3);
    }

    Ok(())
}
