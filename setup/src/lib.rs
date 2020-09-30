#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(trace_macros)]

extern crate hardware;
extern crate multiprocess;
extern crate multiboot;
extern crate alloc;
#[macro_use]
extern crate stdx;

pub mod interrupts;
pub mod globals;