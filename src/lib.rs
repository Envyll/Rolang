#![allow(non_upper_case_globals)]

use imstr::string::ImString;
use std::rc::Rc;

pub mod generation;
pub mod lexer;
pub mod normalizer;
pub mod parser;
pub mod semantics;

type ImStrData = Rc<String>;
pub type ImStr = ImString<ImStrData>;

pub use anyhow::Result;
use intmap::{IntKey, IntMap};
use smallvec::SmallVec;

pub type AResult<T = ()> = Result<T>;

pub trait Multimap<Key> {
    type Value;

    fn insert_value(&mut self, key: Key, value: Self::Value);
}

impl<Key: IntKey, T, const len: usize> Multimap<Key> for IntMap<Key, SmallVec<T, len>> {
    type Value = T;
    fn insert_value(&mut self, key: Key, value: T) {
        if let Some(val) = self.get_mut(key) {
            val.push(value);
        } else {
            self.insert(key, [value].into());
        }
    }
}
