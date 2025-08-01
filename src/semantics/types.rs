use crate::ImStr;
use crate::parser::TokenInd;
use im::{HashMap, HashSet, Vector};
use std::rc::Rc;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    No,
    /// `s64`
    Type,
    /// `(0s64 const)`
    Constant(Constant),
    /// `mod (def1 := …; …)`
    Module(/*todo*/),
    /// `dyn`
    Dyn(/*todo*/),
    /// `(Box::default)^`
    Undefined(/*todo*/),
    Lifetime(Lifetime),

    /// `3u32`
    Integer {
        size: u32,
        signed: Option<bool>,
    },
    /// `3f32`
    Float(FPType),
    /// `(t: t)`
    Function {
        return_type: Rc<Type>,
        params: Vector<Type>,
        realized: usize,
    },
    /// `type::[Type1; Type2; …; id1: IdType1; id2: IdType2; …]`
    Tuple(Vector<Type>, HashMap<ImStr, Type>),
    /// `Arr::[]`
    Array(Rc<Type>, Option<u64>),
    /// `type …`
    Struct(StructType, Module),
    /// `u32 || []` — union types
    Either(HashSet<Type>),
    /// `1 'a::@@` — reference to any value
    Ref {
        ty: Rc<Type>,
        mutability: Option<bool>,
        lifetime: Lifetime,
    },
    /// `"abc" .box copy` — lifetimed copy of any value
    Copy {
        ty: Rc<Type>,
        lifetime: Lifetime,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructType {
    /// `type`
    Single,
    /// `type::[Type1; Type2; …; id1: IdType1; id2: IdType2; …]`
    Tuple(Vector<Type>, HashMap<ImStr, Type>),
    /// `type::(opt1; opt2; …)`
    Enum(Vector<StructType>),
    /// `type::id par1 par2 …`
    Function {
        name: ImStr,
        args: Vector<Type>,
        realized: usize,
    },
    /// `super.type::opt `(`=> super_opt`)?
    Class {
        parent: Rc<Type>,
        fields: HashMap<ImStr, Type>,
        hidden_fields: Vector<Type>,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DefPlace {
    pub ind: TokenInd,
    pub file: ImStr,
    pub project: ImStr,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lifetime {
    DefPlace(DefPlace),
    Static,
    Unsafe,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module {
    pub depends_on: HashMap<ImStr, Type>,
    /// None if anonymous (tuple, object)
    pub def_place: Option<DefPlace>,
    pub constants: HashMap<ImStr, Constant>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FPType {
    //todo
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Constant {
    //todo
}
