use derivative::Derivative;
use derive_more::{IsVariant, TryUnwrap, Unwrap};
use inkwell::context::ContextRef;
use inkwell::types::BasicTypeEnum;

#[non_exhaustive]
#[derive(Clone, Debug, Derivative, PartialEq, TryUnwrap, Unwrap, IsVariant)]
#[derivative(Default)]
pub enum RoType {
    Never,
    Int,
    #[derivative(Default)]
    Tuple(Vec<RoType>)
}

impl RoType {
    pub fn unit() -> RoType {
        RoType::Tuple(Vec::new())
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, RoType::Tuple(n) if n.is_empty())
    }

    pub fn basic_type<'a>(&self, ctx: ContextRef<'a>) -> BasicTypeEnum<'a> {
        match self {
            RoType::Never => {ctx.struct_type(&[], false).into()}
            RoType::Int => {ctx.i64_type().into()}
            RoType::Tuple(t) => {ctx.struct_type(&t.iter().map(|t|t.basic_type(ctx)).collect::<Vec<_>>(), false).into()}
        }
    }
}