#![feature(asm)]
#![feature(const_fn)]
#![feature(allocator_api)]
#![no_std]

#[macro_use]
extern crate bitflags;
extern crate multiboot;
extern crate stdx;
extern crate hardware;
extern crate stdx_memory;
extern crate display;

pub mod frame;
pub mod paging;
pub mod allocator;