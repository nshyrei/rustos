#![feature(lang_items)]
#![feature(asm)]
#![no_std]

extern crate alloc;
extern crate hardware;
extern crate pic8259_simple;

pub mod executor;
pub mod process;
pub mod sync;

use core::any::Any;
use alloc::boxed::Box;

pub type Message = Box<dyn Any>;

pub type ProcessBox = Box<dyn Process>;

pub trait Process {

    fn process_message(&mut self, message : Message) -> ();

    /*fn process_message_box(&mut self, message : Message) -> Box<FnMut(Self, Message)> {
        Box::new(Process::process_message)
    }*/

}