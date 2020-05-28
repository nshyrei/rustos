pub mod idt;
pub mod handler;
pub mod pic;

use ::x86_64::interrupts::idt::InterruptTable;

#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli" :::: "volatile");
    }
}

#[inline(always)]
pub fn enable_interrupts() {
    unsafe {
        asm!("sti" :::: "volatile");
    }
}

pub unsafe fn load_interrupt_table(table : &InterruptTable){
    let ptr = &table.pointer();

    asm!("lidt ($0)" :: "r" (ptr) : "memory")
}

use core::ptr;
use pic8259_simple::ChainedPics;

pub struct InterruptTableHelp {
    pub value : Option<ptr::NonNull<InterruptTable>>
}

