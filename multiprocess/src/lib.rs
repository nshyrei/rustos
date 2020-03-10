#![feature(lang_items)]
#![feature(asm)]
#![no_std]

extern crate alloc;

pub mod executor;
pub mod process;

use core::any::Any;
use alloc::boxed::Box;
use alloc::rc::Rc;

pub type Message = Box<dyn Any>;

pub type ProcessBox = Box<dyn Process>;

pub trait Process {

    fn process_message(&mut self, message : Message) -> ();
}