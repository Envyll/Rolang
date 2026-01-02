// didnt work :( — useless



/*
use std::error::Error;
use melior::Context;
use melior::dialect::{func, arith, DialectRegistry};
use melior::ir::*;
use melior::ir::attribute::{StringAttribute, TypeAttribute};
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;
use melior::utility::register_all_dialects;

type Result<T, E = Box<dyn Error>> = core::result::Result<T, E>;

#[test]
fn test() -> Result<()> {
    let registry = DialectRegistry::new();
    register_all_dialects(&registry);

    let context = Context::new();
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();

    let location = Location::unknown(&context);
    let module = Module::new(location);

    let index_t = Type::index(&context);

    module.body().append_operation(
        func::func(
            &context,
            StringAttribute::new(&context, "add"),
            TypeAttribute::new(
                FunctionType::new(&context, &[index_t, index_t], &[index_t]).into(),
            ),
            {
                let block = Block::new(&[(index_t, location), (index_t, location)]);
                
                let sum = block
                    .append_operation(arith::addi(
                        block.argument(0)?.into(),
                        block.argument(1)?.into(),
                        location,
                    ))
                    .result(0)?;
                
                
                block.append_operation(func::r#return(&[sum.into()], location));

                let region = Region::new();
                region.append_block(block);
                region
            },
            &[],
            location
        )
    );

    assert!(module.as_operation().verify());

    Ok(())
}
*/