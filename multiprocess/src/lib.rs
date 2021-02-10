#![feature(lang_items)]
#![feature(asm)]
#![no_std]

extern crate alloc;
extern crate hardware;
extern crate memory;

pub mod executor;
pub mod process;
pub mod sync;

use core::mem;
use hardware::x86_64::interrupts::handler::InterruptStackFrameValue;
use hardware::x86_64::registers;

