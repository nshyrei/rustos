#![no_std]
#![feature(abi_x86_interrupt)]

extern crate hardware;
extern crate multiprocess;
extern crate multiboot;

pub mod interrupts;
pub mod globals;