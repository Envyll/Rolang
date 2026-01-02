use inkwell::context::{Context};
use inkwell::{AddressSpace, OptimizationLevel};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::values::{GenericValue};
use rolang::Result;




// test if llvm is loaded ok
// + i'm trying to see how to work with this (lol)

#[test]
fn test_interpret() -> Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;


    let context = Context::create();
    let module = context.create_module("some_module");

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple)?;
    let target_machine = target.create_target_machine(
        &triple,
        "generic",
        "",
        OptimizationLevel::None,
        RelocMode::Default,
        CodeModel::JITDefault,
    ).ok_or("no target machine")?;

    module.set_triple(&triple);
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());

    let builder = context.create_builder();


    let ty = context.i64_type();
    let ty32 = context.i32_type();
    let ptr = context.ptr_type(AddressSpace::default());
    let f_ty = ty.fn_type(&[ptr.into()], false);
    let xy_ty = context.struct_type(&[ty.into(), ty.into()], false);

    let aaaa = module.add_function("aaaa", f_ty, None);
    let block = context.append_basic_block(aaaa, "");

    builder.position_at_end(block);

    let p = aaaa.get_nth_param(0).ok_or("")?.into_pointer_value();

    let x = unsafe { builder.build_in_bounds_gep(xy_ty, p, &[ty.const_zero(), ty32.const_int(0, false)],  "") }?;
    let y = unsafe { builder.build_in_bounds_gep(xy_ty, p, &[ty.const_zero(), ty32.const_int(1, false)],  "") }?;

    let x_s = builder.build_load(ty, x, "")?.into_int_value();
    let y_s = builder.build_load(ty, y, "")?.into_int_value();



    let shift = builder.build_left_shift(y_s, ty.const_int(1, false), "")?;
    let diff = builder.build_int_nuw_sub(shift, x_s, "")?;

    //let ptr_diff = builder.build_ptr_diff(ty, y, x, "")?;

    let _instr = builder.build_return(Some(&diff))?;

    module.print_to_stderr();
    module.verify()?;

    let mut xy = (19u64, 10u64);



    let engine = module.create_interpreter_execution_engine()?;

    let z = unsafe{engine.run_function(aaaa, &[
        &GenericValue::create_generic_value_of_pointer(&mut xy),
    ]) };


    dbg!(xy);
    assert_eq!(dbg!(z.as_int(false)), 1);

    Ok(())
}


#[test]
fn test_jit() -> Result<()> {
    Target::initialize_native(&InitializationConfig::default())?;
    
    let context = Context::create();
    let module = context.create_module("some_module");
    let builder = context.create_builder();


    let ty64 = context.i64_type();
    let ty8 = context.i8_type();
    let f_ty = ty8.fn_type(&[ty64.into()], false);

    let f = module.add_function("f", f_ty, None);
    let block = context.append_basic_block(f, "");

    let block_0 = context.append_basic_block(f, "zero");
    let block_1 = context.append_basic_block(f, "one");
    let block_else = context.append_basic_block(f, "else");
    
    builder.position_at_end(block);
    
    let arg = f.get_first_param().ok_or("arg err")?.into_int_value();
    
    let _ = builder.build_switch(
        arg,
        block_else,
        &[
            (ty64.const_zero(), block_0),
            (ty64.const_int(1, false), block_1),
        ]
    )?;
    
    builder.position_at_end(block_0);
    let _instr = builder.build_return(Some(&ty8.const_int(1, false)))?;

    builder.position_at_end(block_1);
    let _instr = builder.build_return(Some(&ty8.const_int(0, false)))?;
    
    builder.position_at_end(block_else);
    let coal = builder.build_int_cast(arg, ty8, "coal")?;
    let _instr = builder.build_return(Some(&coal))?;

    module.print_to_stderr();
    module.verify()?;
    
    let engine = module.create_jit_execution_engine(OptimizationLevel::Less)?; 


    let f = unsafe{engine.get_function::<unsafe extern "C" fn (_) -> u8>("f") }?;


    assert_eq!(dbg!(unsafe {f.call(0)}), 1);
    assert_eq!(dbg!(unsafe {f.call(1)}), 0);
    assert_eq!(dbg!(unsafe {f.call(0x103)}), 3);

    Ok(())
}
