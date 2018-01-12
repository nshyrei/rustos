#![feature(nonzero)]
#![no_std]

extern crate stdx;

pub mod smart_ptr;
pub mod heap;
pub mod collections;


use core::mem;

pub trait MemoryAllocator {
    
    /*
    fn from_address(address: usize, size : usize) -> Self;

    fn from_address_for_type<T>(address: usize) -> Self
    where Self : core::marker::Sized {
        use core::mem::size_of;
        Self::from_address(address, size_of::<T>())
    }

    */
    fn allocate(&mut self, size : usize) -> Option<usize>;

    fn allocate_for<T>(&mut self) -> Option<usize> {
        self.allocate(mem::size_of::<T>())        
    }    

    fn free(&mut self, pointer : usize);

    /*
    fn start_address(&self) -> usize;

    fn end_address(&self) -> usize;
    */
}

pub trait ConstantSizeMemoryAllocator {    

    
    /*
    fn from_address(address: usize, size : usize, allocation_size : usize) -> Self;

    fn from_address_for_type<T>(address: usize, size : usize) -> Self
    where Self : core::marker::Sized {        
        Self::from_address(address, size, mem::size_of::<T>())
    }

    fn from_address_for_type_multiple<T>(address: usize, elems_count : usize) -> Self
    where Self : core::marker::Sized {    
        let elem_size = mem::size_of::<T>();
        Self::from_address(address, elem_size * elems_count, elem_size)
    }
    */

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