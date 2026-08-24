use crate::semantics::ro_type::RoType;
use inkwell::context::ContextRef;
use inkwell::types::{BasicType, BasicTypeEnum};
use itertools::Itertools;

impl RoType {
    pub fn basic_type<'a>(&self, ctx: ContextRef<'a>) -> BasicTypeEnum<'a> {
        match self {
            RoType::Unit => ctx.struct_type(&[], false).as_basic_type_enum(),
            RoType::Int => ctx.i64_type().as_basic_type_enum(),
            RoType::Tuple(values) => {
                let new = values.0.basic_type(ctx);
                let old = values.1.basic_type(ctx).into_struct_type();
                let tys = old.get_field_types_iter();
                ctx.struct_type(&[new].into_iter().chain(tys).collect_vec(), false)
                    .into()
            }
            RoType::MaybeUnknown(val) => val.0.get().map(|this| this.basic_type(ctx)).unwrap(),
        }
    }
}
