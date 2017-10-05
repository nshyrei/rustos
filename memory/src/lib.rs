#![feature(asm)]
#![feature(const_fn)]
#![no_std]

#[macro_use]
extern crate bitflags;
extern crate multiboot;
extern crate stdx;
extern crate hardware;

pub mod kernel;
pub mod frame;
pub mod paging;

pub const HEAP_START: usize = 0x20000000; //start at 512 mb, move to somewhere constant!!!
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub fn zero_frame(frame : &frame::Frame) {
    use core::ptr;
    use frame::FRAME_SIZE;

    unsafe { ptr::write(frame.address() as *mut [u8; FRAME_SIZE], [0; FRAME_SIZE]); }
}