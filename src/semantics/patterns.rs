use crate::semantics::types::RoType;
use crate::AResult;
use derive_more::From;
use inkwell::builder::Builder;
use inkwell::values::BasicValueEnum;
use smallbitvec::SmallBitVec;
use std::borrow::Borrow;
use std::collections::BTreeMap;

pub type PatternSeq<T> = Vec<T>;
pub type BoolSeq = SmallBitVec;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum PatternColumnData {
    Int{
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
            let val = seq[i];
            seq.push(val);
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
#[derive(Debug, Clone, From, Eq, PartialEq, Ord, PartialOrd, Hash, )]
pub enum PatternColumnSubposition {
    #[from]
    Field(usize),
}

pub type PatternColumnPosition = Vec<PatternColumnSubposition>;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, )]
pub struct PatternColumnOrder {
    /// # note
    /// should be 0 until matching table building finished
    pub priority: usize,
    pub position: PatternColumnPosition,
}

#[derive(Debug, Clone, Default)]
pub struct PatternTableBuilder {
    pub elements: BTreeMap<PatternColumnPosition, PatternColumnData>,
    pub branches: Vec<usize>
}

#[derive(Debug, Clone, Default)]
pub struct PatternTable {
    pub elements: BTreeMap<PatternColumnOrder, PatternColumnData>,
    pub branches: Vec<usize>
}

impl PatternTableBuilder {
    pub fn clone_last_until<T>(&mut self, count: usize, bound: &T) where T: Ord, PatternColumnPosition: Borrow<T> {
        self.elements.range_mut(..bound).for_each(|(_, v)| v.clone_last(count) );
        self.elements.range_mut(bound..).for_each(|(_, v)| v.push_empty(count) );
        extend_cloned!(&mut self.branches, count);
    }

    pub fn push_new(&mut self, branch: usize) {
        self.branches.push(branch);
        self.elements.iter_mut().for_each(|(_, v)| v.push_empty(1)) ;
    }

    pub fn build(self) -> PatternTable {
        let Self{branches, elements} = self;

        let elements : BTreeMap<_, _> = elements.into_iter()
            .map(|(position, data)| (PatternColumnOrder{priority: data.priority(), position}, data))
            .collect();


        PatternTable{branches, elements}
    }
}

impl PatternColumnOrder {
    pub fn take_value<'a, 'b>(&self, mut val: BasicValueEnum<'a>, builder: &Builder<'a>, mut ty: &'b RoType) -> AResult<BasicValueEnum<'a>> {

        for pos in &self.position {
            (val, ty) = match pos {
                &PatternColumnSubposition::Field(pos) => match ty {
                    RoType::Never => {(val, ty)}
                    RoType::Int => {unreachable!()}
                    RoType::Tuple(tys) => {
                        (builder.build_extract_value(val.into_struct_value(), pos as u32, "")?, &tys[pos])
                    }
                }
            }
        }

        Ok(val)
    }
}