use crate::semantics::constants_values::ConstantValue;
use crate::semantics::modules::ModuleInd;
use crate::ImStr;
use im::{HashMap, HashSet, Vector};
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Type {
    /// Value example: `3`\
    /// Type example: `i64`
    Integer {
        size: u32,
        signed: Option<bool>,
    },
    /// Value example: `3f32`\
    /// Type example: `f32`
    Float(FPType),
    /// Value example: `((t1 as u32) (t2 as u32) (t3 as u32) as u32: t1 + t2 * t3)`\
    /// Type example: `fn u32 => [t1: u32].fn u32 => [t1: u32].[t2: u32].fn u32 => u32`
    Function {
        base: Option<Rc<Type>>,
        arg: Rc<Type>,
        ty: Rc<Type>,
    },
    /// Value example: `Base::[].[Type1::[]; Type2::[]; id1: IdType1::[]; id2: IdType2::[];]`\
    /// Type example: `Base.[Type1; Type2; id1: IdType1; id2: IdType2;]`
    Tuple(Option<Rc<Type>>, Vector<Rc<Type>>, HashMap<ImStr, Type>),
    /// Value example: `Arr::[1; 2]`\
    /// Type example: `Arr[2] => i64`
    Array(Rc<Type>, Option<u64>),
    /// Union type\
    /// Value example: `[] .either`\
    /// Type example: `Either || u32 || []`
    Either(HashSet<Rc<Type>>),
    /// Reference to any value\
    /// Value example: `1 mut@`\
    /// Type example: `mut@ => i64`
    Ref {
        ty: Rc<Type>,
        mutability: bool,
        lifetime: Lifetime,
        allocator: Option<Rc<Type>>,
    },
    /// Lifetimed copy\
    /// Value example: `"abc" @'box const~`\
    /// Type example: `const~ => 'box@ => Str`
    Copy {
        ty: Rc<Type>,
        mutability: bool,
        lifetime: Lifetime,
    },
    Generic(ImStr, TypeTraits),
    Struct {ty: ModuleInd, params: HashMap<ImStr, ConstantValue>},
    Type
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lifetime {
    Local(u32),
    Static,
    Unsafe,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FPType {
    F16, BF16, F32, F64, F128,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeTraits{
    copy_borrowable: Option<bool>,
    //todo
}
