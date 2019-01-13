#![feature(nonzero)]
#![feature(type_ascription)]
#![no_std]

extern crate stdx;

pub mod heap;
pub mod collections;
pub mod trees;

use core::mem;

pub trait MemoryAllocator {

    fn allocate(&mut self, size : usize) -> Option<usize>;

    fn allocate_for<T>(&mut self) -> Option<usize> {
        self.allocate(mem::size_of::<T>())        
    }    

    fn free(&mut self, pointer : usize);

    fn assigned_memory_size() -> usize;

    fn aux_data_structures_size() -> usize;
}

pub trait ConstantSizeMemoryAllocator {

    fn allocate_size(&mut self) -> Option<usize>;    

    fn free_size(&mut self, pointer : usize);

    fn assigned_memory_size() -> usize;

    fn aux_data_structures_size() -> usize;
}

impl<T> MemoryAllocator for T where T: ConstantSizeMemoryAllocator
{
    fn allocate(&mut self, size : usize) -> Option<usize> {
        self.allocate_size()
    }

    fn free(&mut self, pointer : usize) {
        self.free_size(pointer)
    }

    fn assigned_memory_size() -> usize {
        T::assigned_memory_size()
    }

    fn aux_data_structures_size() -> usize {
        T::aux_data_structures_size()
    }
}