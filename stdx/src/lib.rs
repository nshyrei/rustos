#![no_std]

pub mod iterator;
pub mod util;
pub mod monoid;
pub mod math;
pub mod sequence;

use core::iter;
use core::mem;

pub trait Iterable {
    type Item;
    
    type IntoIter: iter::Iterator<Item=Self::Item>;

    fn iterator(&self) -> Self::IntoIter;
}

pub trait Sequence : Iterable {
    fn length(&self) -> usize;

    fn mem_size_for(length : usize) -> usize {
        Self::cell_size() * length
    }

    fn mem_size(&self) -> usize {
        Self::cell_size() * self.length()
    }

    fn cell_size() -> usize {
        mem::size_of::<Self::Item>()
    }
}

pub trait Map {

    type Key;

    type Value;

    fn contains_key(&self, key : Self::Key) -> bool;

    fn misses_key(&self, key : Self::Key) -> bool {
        !self.contains_key(key)
    }

    fn get_value(&self, key : Self::Key) -> Option<Self::Value>;
}

pub trait BiMap : Map {

    
}