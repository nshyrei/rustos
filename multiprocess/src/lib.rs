#![feature(lang_items)]
#![feature(asm)]
#![no_std]

extern crate alloc;
extern crate hardware;
extern crate memory;
extern crate pic8259_simple;

pub mod executor;
pub mod process;
pub mod sync;

use core::any::Any;
use core::default::Default;
use alloc::boxed::Box;

pub type Message = Box<dyn Any>;

pub type ProcessBox = Box<dyn Process>;

pub trait Process {



    /*unsafe fn before_process(&mut self, message : Message) {
        use core::ptr;
let self_addr = self as *mut _ as u64;
        ptr::write(2961408 as *mut u64, self_addr);
        ptr::write(2961416 as *mut Message, message);

        // stack now points to process stack
        //registers::sp_write(2961408 as u32);

        // reread &self and message from new stack
        let mut new_self = ptr::read::<*mut Self>(2961408 as *const *mut Self);
        let new_message = ptr::read::<Message>(2961416 as *const Message);

        new_self.as_ref().unwrap().process_message(new_message);
    }*/

    fn process_message(&mut self, message : Message) -> ();

    /*fn process_message_box(&mut self, message : Message) -> Box<FnMut(Self, Message)> {
        Box::new(Process::process_message)
    }*/

}