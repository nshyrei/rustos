#![feature(lang_items)]
#![feature(asm)]
#![feature(step_by)]
#![feature(nonzero)]
#![no_std]

pub mod smart_ptr;
pub mod heap;
pub mod collections;

pub trait MemoryAllocator {
    fn allocate(&mut self, size : usize) -> Option<usize>;

    fn allocate_for<T>(&mut self) -> Option<usize> {
        use core::mem;
        self.allocate(mem::size_of::<T>())        
    }

    fn free(&mut self, pointer : usize);

    fn start_address(&self) -> usize;

    fn end_address(&self) -> usize;
}