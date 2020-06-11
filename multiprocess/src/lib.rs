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


use hardware::x86_64::interrupts::handler::InterruptStackFrameValue;
use hardware::x86_64::registers;

pub fn switch_to_running_process(new_process : &executor::ProcessDescriptor, interrupted: &mut InterruptStackFrameValue) {
    let new_process_registers = new_process.registers();

    interrupted.instruction_pointer = new_process_registers.instruction_pointer;
    interrupted.stack_pointer           = new_process_registers.stack_pointer;
    interrupted.cpu_flags                 = new_process_registers.cpu_flags;
}

pub unsafe fn start_new_process(new_process : &mut executor::ProcessDescriptor) {

    let stack_address = new_process.stack_address() as u32;
    let to_write = new_process as *const _ as u64;

    core::ptr::write_unaligned((stack_address - 8) as *mut u64, to_write);

    let process_stack_start = stack_address;

    registers::sp_write(process_stack_start);

    let stack_address_reread = registers::sp_read() - 8;

    let descriptor_address = *(stack_address_reread as *mut u64);

    let descriptor_ptr = core::mem::transmute::<u64, &mut executor::ProcessDescriptor>(descriptor_address);

    descriptor_ptr.process_front_message();

    loop{}
}


