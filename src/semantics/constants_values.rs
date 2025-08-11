use std::rc::Rc;
use im::{HashMap, HashSet, Vector};
use num_bigint::BigInt;
use crate::ImStr;
use crate::parser::ExprInd;
use crate::semantics::modules::ModuleInd;
use crate::semantics::types::{FPType, Lifetime, Type};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ConstantValue {
    Integer{
        value: BigInt,
        size: u32,
        signed: Option<bool>,
    },
    Float{
        value: u128,
        ty: FPType,
    },
    Function {
        expr: ExprInd,
        base: Option<Rc<ConstantValue>>,
        arg: Rc<Type>,
        ty: Rc<Type>,
    },
    Tuple{
        base: Rc<ConstantValue>,
        unnamed: Vec<Rc<ConstantValue>>,
        named: HashMap<ImStr, Rc<ConstantValue>>,
    },
    Array {
        value: Vector<Rc<ConstantValue>>,
        ty: Rc<Type>,
        size: Option<u64>,
    },
    Either {
        value: Rc<ConstantValue>,
        types: HashSet<Rc<ConstantValue>>,
    },
    Ref {
        value: Rc<ConstantValue>,
        mutability: bool,
        lifetime: Lifetime,
        allocator: Option<Rc<Type>>,
    },
    Copy {
        value: Rc<ConstantValue>,
        mutability: bool,
        lifetime: Lifetime,
    },
    Struct {
        base: Option<Rc<Type>>,
        option: ImStr,
        unnamed: Vector<Rc<ConstantValue>>,
        named: HashMap<ImStr, Rc<ConstantValue>>,
        ty: ModuleInd,
        params: HashMap<ImStr, ConstantValue>,
    },
    Type (Option<Rc<Type>>),
}
