
#[derive(Debug, Clone)]
pub enum Pattern {
    Number(u64),
    Or(Vec<Self>),


    Bad
}