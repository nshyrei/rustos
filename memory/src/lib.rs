#![feature(asm)]
#![feature(const_fn)]
#![no_std]

//#![feature(allocator)]
//#![allocator]

#[macro_use]
extern crate bitflags;
extern crate multiboot;
extern crate stdx;
extern crate hardware;

pub mod kernel;
pub mod frame;
pub mod paging;
pub mod heap;

/*
    kernel memory layout

    *---kernel code + stack---**--- frame allocator data structures---**---kernel heap---*
*/