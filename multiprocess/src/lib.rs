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

use core::mem;
use hardware::x86_64::interrupts::handler::InterruptStackFrameValue;
use hardware::x86_64::registers;

/// Switches execution to previously stopped process.
/// # Arguments
///  `next_process` - descriptor of the process to switch to
///  `interrupted` - meta data of the stopped process
pub fn switch_to_running_process(next_process : &executor::ProcessDescriptor, interrupted: &mut InterruptStackFrameValue) {
    let next_process_registers = next_process.registers();

    // New values for CS, SP and FLAGS registers will be picked automatically from `InterruptStackFrameValue` by the processor after exiting the interrupt handler
    // The only thing we need to do here is to populate `interrupted` with the info of process to switch to.
    interrupted.instruction_pointer = next_process_registers.instruction_pointer;
    interrupted.stack_pointer           = next_process_registers.stack_pointer;
    interrupted.cpu_flags                 = next_process_registers.cpu_flags;
}

/// Starts new process.
/// # Arguments
///  `next_process` - descriptor of the process to start
///  # Safety
/// Unsafe because starting new process involves unsafe memory operations
pub unsafe fn start_new_process(new_process : &mut executor::ProcessDescriptor) {
    // What this does is:
    // 1) pushes new process descriptor into new process stack
    // 2) switches SP to point to new process stack, essentially changing context of the current process to new process
    // 3) rereads new process descriptor from new stack
    // 4) calls process `process_message` function which now works in new process environment

    let stack_address           = new_process.stack_address() as u32;
    let descriptor_address   = new_process as *const _ as u64;

    // push process descriptor pointer into new process stack
    core::ptr::write_unaligned((stack_address - (mem::size_of::<u64>() as u32)) as *mut u64, descriptor_address);

    // switches stack pointer to point to new process stack
    // WARNING! After this line ALL local variables above will have undefined values and cannot be used anymore.
    registers::sp_write(stack_address);

    // reread stack head from register, because `stack_address` variable at the top will be undefined
    let descriptor_address = registers::sp_read() - (mem::size_of::<u64>() as u32);

    // read descriptor pointer from new process stack, because variables `descriptor_address` and `new_process` will also be undefined
    let descriptor_pointer_raw = *(descriptor_address as *mut u64);

    let new_process0 = mem::transmute::<u64, &mut executor::ProcessDescriptor>(descriptor_pointer_raw);

    // execute process code
    new_process0.run_process();

    loop{}
}


