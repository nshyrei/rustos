#![feature(nonzero)]
#![feature(type_ascription)]
#![no_std]

extern crate stdx;
extern crate display;

pub mod heap;
pub mod collections;
pub mod trees;

use core::mem;

pub trait MemoryAllocatorMeta {
    /// start address of memory this allocator works with
    /// # Warning:
    /// Computing end_address() -  start_address() won't correspond to a size of memory used to allocate data
    fn start_address(&self) -> usize;

    /// end address of memory this allocator works with
    /// # Warning:
    /// Computing end_address() -  start_address() won't correspond to a size of memory used to allocate data
    fn end_address(&self) -> usize;

    fn full_size(&self) -> usize {
        self.end_address() - self.start_address() + 1
    }

    /// Memory size for data allocation
    fn assigned_memory_size(&self) -> usize {
        self.full_size() - self.aux_data_structures_size()
    }

    fn object_allocation_start_address(&self) -> usize {
        self.start_address() + self.aux_data_structures_size() + 1
    }

    /// Memory size for internal allocator data structures
    fn aux_data_structures_size(&self) -> usize;
}

pub trait MemoryAllocator : MemoryAllocatorMeta {

    fn allocate(&mut self, size : usize) -> Option<usize>;

    fn allocate_for<T>(&mut self) -> Option<usize> {
        self.allocate(mem::size_of::<T>())        
    }    

    fn free(&mut self, pointer : usize);
}

// Allocator that can only allocate/free constant size blocks of memory
pub trait ConstantSizeMemoryAllocator : MemoryAllocatorMeta {

    fn allocate_size(&mut self) -> Option<usize>;    

    fn free_size(&mut self, pointer : usize);
}

impl<T> MemoryAllocator for T where T: ConstantSizeMemoryAllocator
{
    fn allocate(&mut self, size : usize) -> Option<usize> {
        self.allocate_size()
    }

    fn free(&mut self, pointer : usize) {
        self.free_size(pointer)
    }
}
