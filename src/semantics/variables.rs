use std::cell::OnceCell;
use std::rc::Rc;
use derivative::Derivative;
use crate::ImStr;
use crate::parser::ExprInd;
use crate::semantics::patterns::Pattern;
use crate::semantics::types::RoType;
use crate::semantics::VariablesContext;

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, Hash, PartialEq)]
pub struct FunctionDefinition {
    pub expr: ExprInd,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub args: VariablesContext,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub return_type: Rc<OnceCell<RoType>>,
}

#[derive(Debug, Clone, Default, Derivative)]
#[derivative(Eq, Hash, PartialEq)]
pub enum VariableDefinition{
    #[default]
    None,
    Function(ImStr),
    Variable{
        name: ImStr,
        place: ExprInd,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        n: u32,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ty: RoType,
    },
    Special(ImStr),
    Number(u64),

    Match(
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        Vec<(Pattern, ExprInd)>,

        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        RoType
    ),

    Tuple(
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        Vec<RoType>
    ),
    Member(u32),

    Unknown,
}

impl VariableDefinition {
    pub fn ty(&self) -> RoType{
        match self {
            VariableDefinition::None => {
                unimplemented!()
            }
            VariableDefinition::Function(_) => {
                unimplemented!()
            }
            VariableDefinition::Variable { ty, .. } => {
                ty.clone()
            }
            VariableDefinition::Special(_) => {
                unimplemented!()
            }
            VariableDefinition::Number(_) => {
                RoType::Int
            }
            VariableDefinition::Match(_, ty) => {
                ty.clone()
            }
            VariableDefinition::Tuple(ty) => {
                RoType::Tuple(ty.clone())
            }
            VariableDefinition::Unknown => {
                unimplemented!()
            }
            VariableDefinition::Member(_) => {
                unimplemented!()
            }
        }
    }
}