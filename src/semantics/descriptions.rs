use crate::ImStr;
use crate::parser::ExprInd;
use crate::semantics::BindingReference;
use crate::semantics::patterns::PatternTable;
use crate::semantics::ro_type::RoType;
use derive_more::{TryUnwrap, Unwrap};
use std::rc::Rc;

#[non_exhaustive]
#[derive(Debug, Clone, Default, TryUnwrap, Unwrap)]
#[try_unwrap(ref, ref_mut)]
#[unwrap(ref, ref_mut)]
pub enum ExpressionDescription {
    #[default]
    None,
    #[try_unwrap(ignore)]
    #[unwrap(ignore)]
    Function {
        name: ImStr,
        input: Rc<[RoType]>,
        output: RoType,
    },
    Variable(BindingReference),
    Special(ImStr),
    Number(u64),
    Unit,
    Tuple(RoType),
    Member(u32),

    #[try_unwrap(ignore)]
    #[unwrap(ignore)]
    Let {
        patterns: PatternTable,
        pred_ty: RoType,
    },
    #[try_unwrap(ignore)]
    #[unwrap(ignore)]
    Match {
        branches: Vec<ExprInd>,
        patterns: PatternTable,
        ty: RoType,
        pred_ty: RoType,
    },

    Unknown,
}

impl ExpressionDescription {
    pub fn ty(&self) -> RoType {
        match self {
            ExpressionDescription::None => {
                unimplemented!()
            }
            ExpressionDescription::Function { output, .. } => output.clone(),
            ExpressionDescription::Variable(desc) => desc.ty.clone(),
            ExpressionDescription::Special(_) => {
                unimplemented!()
            }
            ExpressionDescription::Number(_) => RoType::Int,
            ExpressionDescription::Match { ty, .. } => ty.clone(),
            ExpressionDescription::Let { .. } => RoType::Unit,
            ExpressionDescription::Unit => RoType::Unit,
            ExpressionDescription::Tuple(ty) => ty.clone(),
            ExpressionDescription::Unknown => {
                unimplemented!()
            }
            ExpressionDescription::Member(_) => {
                unimplemented!()
            }
        }
    }
}
