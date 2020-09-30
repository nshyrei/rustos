#![no_std]
pub mod iterator;
pub mod util;
pub mod monoid;
pub mod math;
pub mod sequence;
pub mod macros;

use core::iter;
use core::mem;

// Describes a type that can be iterated over
pub trait Iterable {
    type Item;
    
    type IntoIter: iter::Iterator<Item=Self::Item>;

    fn iterator(&self) -> Self::IntoIter;
}

// Describes a type represents a finite sequence of elements
pub trait Sequence : Iterable {

    // Length of this sequence
    fn length(&self) -> usize;

    // How much memory is needed to hold `length` number of elements
    fn mem_size_for(length : usize) -> usize {
        Self::cell_size() * length
    }

    // How much memory is needed to hold all the elements of this sequence
    fn mem_size(&self) -> usize {
        Self::mem_size_for(self.length())
    }

    // How much memory is needed to hold one element
    fn cell_size() -> usize {
        mem::size_of::<Self::Item>()
    }
}