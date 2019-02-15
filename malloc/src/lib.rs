#![no_std]


#![feature(allocator_api)]
#![feature(const_fn)]

use core::alloc::{Alloc, AllocErr, Layout, GlobalAlloc};
use core::ptr::NonNull;
use core::ptr::null_mut;

pub struct TestAllocator {

}

impl TestAllocator {
    pub const fn new() -> Self {
        TestAllocator {}
    }
}

unsafe impl<'a> Alloc for &'a TestAllocator {

    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let a = 10;
        let b = a;

        Err(AllocErr)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let a = 10;
        let b = a;
    }    
}

unsafe impl GlobalAlloc for TestAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 { 
        let a = 10;
        let b = a;
null_mut()
        }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
               let a = 10;
        let b = a;
 
    }
}
