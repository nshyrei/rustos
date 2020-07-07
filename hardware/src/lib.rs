#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![feature(const_fn)]
#![no_std]

#[macro_use]
extern crate bitflags;
extern crate pic8259_simple;

pub mod x86_64;