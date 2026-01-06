use derivative::Derivative;
use inkwell::context::ContextRef;
use inkwell::types::BasicTypeEnum;

#[derive(Clone, Debug, Derivative)]
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

    pub fn basic_type<'a>(&self, ctx: ContextRef<'a>) -> BasicTypeEnum<'a> {
        match self {
            RoType::Never => {ctx.struct_type(&[], false).into()}
            RoType::Int => {ctx.i64_type().into()}
            RoType::Tuple(t) => {ctx.struct_type(&t.iter().map(|t|t.basic_type(ctx)).collect::<Vec<_>>(), false).into()}
        }
    }
}