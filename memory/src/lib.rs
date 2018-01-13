#![feature(asm)]
#![feature(const_fn)]
#![no_std]

#[macro_use]
extern crate bitflags;
extern crate multiboot;
extern crate stdx;
extern crate hardware;
extern crate stdx_memory;

pub mod util;
pub mod frame;
pub mod paging;
pub mod allocator;

/*
    kernel memory layout. All physical addressess are equal to virtual here.

    *---kernel code + stack---**--- frame allocator data structures---**---heap data structures---*
*/