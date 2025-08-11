use std::cell::RefCell;
use std::rc::Rc;
use derivative::Derivative;
use im::{HashMap, Vector};
use crate::ImStr;
use crate::semantics::Place;
use crate::semantics::types::Type;
use crate::semantics::constants_values::ConstantValue;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Module {
    pub this: ConstantInfo,
    pub params: HashMap<String, Rc<Type>>,
    pub constants: HashMap<String, ConstantInfo>,
    pub struct_type: Option<StructType>,
    pub is_public: bool,
}

#[derive(Derivative)]
#[derivative(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConstantInfo {
    pub name: ImStr,
    pub declaration: Place,
    pub definitions: Vec<Place>,
    pub dependencies: HashMap<ImStr, Rc<Type>>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub value: RefCell<std::collections::HashMap<HashMap<ImStr, ConstantValue>, Rc<ConstantValue>>>,
}

#[derive(Debug, Clone, Default)]
pub struct ModulesTable(
    pub std::collections::HashMap<ModuleInd, Module>
);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModuleInd(pub Vec<ImStr>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TypeMod{
    pub ty: Rc<Type>,
    pub pub_: bool,
    pub mut_: bool,
    pub opt_: bool,
}

/// `type BaseType.([T0; T1; …; id0: TID0; …]; opt1[…]; opt2[…]; …)`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructType {
    pub base: Option<Rc<Type>>,
    pub options: HashMap<String, (Vector<TypeMod>, HashMap<ImStr, TypeMod>)>,
}