use crate::parser::ExprInd;
use crate::semantics::ro_type::RoType;
use crate::{AResult, ImStr};
use derive_more::{From, IsVariant, TryUnwrap, Unwrap};
use inkwell::builder::Builder;
use inkwell::values::BasicValueEnum;
use smallbitvec::SmallBitVec;
use smallvec::{SmallVec, smallvec};
use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap};

pub type PatternSeq<T> = SmallVec<T, 2>;
pub type BoolSeq = SmallBitVec;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum PatternColumnData {
    Int {
        values: PatternSeq<u64>,
        ignore: BoolSeq,
    },
}

macro_rules! extend_cloned {
    ($seq:expr, $count:expr) => {
        let seq = $seq;
        let count = $count;
        seq.reserve(count);

        let end = seq.len();
        let start = end - count;

        for i in start..end {
            seq.push(seq[i].clone());
        }
    };
}

impl PatternColumnData {
    pub fn clone_last(&mut self, count: usize) {
        match self {
            PatternColumnData::Int { values, ignore } => {
                extend_cloned!(values, count);
                extend_cloned!(ignore, count);
            }
        }
    }

    pub fn push_empty(&mut self, count: usize) {
        match self {
            PatternColumnData::Int { values, ignore } => {
                values.extend([0].into_iter().cycle().take(count));
                ignore.extend([true].into_iter().cycle().take(count));
            }
        }
    }

    pub fn priority(&self) -> usize {
        match self {
            PatternColumnData::Int { ignore, .. } => {
                ignore.len() - ignore.iter().filter(|&v| v).count()
            }
        }
    }
}

#[non_exhaustive]
#[derive(
    Debug, Clone, Copy, From, Eq, PartialEq, Ord, PartialOrd, Hash, IsVariant, TryUnwrap, Unwrap,
)]
pub enum PatternColumnSubposition {
    #[from]
    Field(usize),
}

// if not the traverse order, should have chosen a simplier data struct :(
pub type PatternColumnPosition = rpds::Vector<PatternColumnSubposition>;
pub type BranchesVec = SmallVec<usize, 2>;
pub type Bindings = Vec<HashMap</*name*/ ImStr, BindingInfo>>;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct PatternColumnOrder {
    pub priority: usize,
    pub position: PatternColumnPosition,
}

#[derive(Debug, Clone)]
pub struct BindingInfo {
    pub pattern_column_position: PatternColumnOrder,
    pub ty: RoType,
    pub expr: ExprInd,
}

#[derive(Debug, Clone, Default)]
pub struct PatternTableBuilder {
    pub elements: BTreeMap<PatternColumnPosition, PatternColumnData>,
    pub branches: BranchesVec,
    pub bindings: Bindings,
}

#[derive(Debug, Clone, Default)]
pub struct PatternTable {
    pub elements: BTreeMap<PatternColumnOrder, PatternColumnData>,
    pub branches: BranchesVec,
    pub bindings: Bindings,
}

impl PatternTableBuilder {
    pub fn clone_last_until<T>(&mut self, count: usize, bound: &T)
    where
        T: Ord,
        PatternColumnPosition: Borrow<T>,
    {
        self.elements
            .range_mut(..bound)
            .for_each(|(_, v)| v.clone_last(count));
        self.elements
            .range_mut(bound..)
            .for_each(|(_, v)| v.push_empty(count));
        extend_cloned!(&mut self.branches, count);
        extend_cloned!(&mut self.bindings, count);

        self.bindings
            .iter_mut()
            .rev()
            .take(count)
            .for_each(|bindings| {
                let mut exclude_list: SmallVec<_, 1> = smallvec![];
                for (name, binding) in &mut *bindings {
                    if binding.pattern_column_position.position.borrow() >= bound {
                        exclude_list.push(name.clone());
                    }
                }
                for exclude in exclude_list {
                    bindings.remove(&exclude);
                }
            });
    }

    pub fn push_new(&mut self) {
        let branch = self.branches.last().map(|br| br + 1).unwrap_or(0);

        self.branches.push(branch);
        self.elements.iter_mut().for_each(|(_, v)| v.push_empty(1));
        self.bindings.push(HashMap::new());
    }

    pub fn build(self) -> PatternTable {
        let Self {
            branches,
            elements,
            bindings,
        } = self;

        let elements: BTreeMap<_, _> = elements
            .into_iter()
            .map(|(position, data)| {
                (
                    PatternColumnOrder {
                        priority: data.priority(),
                        position,
                    },
                    data,
                )
            })
            .collect();

        PatternTable {
            branches,
            elements,
            bindings,
        }
    }
}

impl PatternColumnOrder {
    pub fn take_value<'a>(
        &self,
        mut val: BasicValueEnum<'a>,
        builder: &Builder<'a>,
        mut ty: &RoType,
    ) -> AResult<BasicValueEnum<'a>> {
        for &(mut pos_n) in &self.position {
            let mut curr_element = 0;
            'repeat: loop {
                break 'repeat (val, ty) = match &mut pos_n {
                    PatternColumnSubposition::Field(pos) => match ty {
                        RoType::Int => {
                            unreachable!()
                        }
                        RoType::Unit => {
                            unreachable!()
                        }
                        RoType::Tuple(values) if *pos == 0 => (
                            builder.build_extract_value(
                                val.into_struct_value(),
                                curr_element,
                                "",
                            )?,
                            &values.0,
                        ),
                        RoType::Tuple(values) => {
                            (ty, *pos, curr_element) = (&values.1, *pos - 1, curr_element + 1);
                            continue 'repeat;
                        }
                        RoType::MaybeUnknown(v) if let Some(inner) = v.0.get() => {
                            ty = inner;
                            continue 'repeat;
                        }
                        RoType::MaybeUnknown(_) => unreachable!(),
                    },
                };
            }
        }

        Ok(val)
    }
}
