#![feature(asm)]
#![feature(llvm_asm)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(const_fn)]
#![no_std]

#[macro_use]
extern crate bitflags;
extern crate bit_field;
extern crate pic8259_simple;


pub mod x86_64;