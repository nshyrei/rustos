use core::fmt::Write;

use hardware::x86_64::registers;
use hardware::x86_64::interrupts;
use hardware::x86_64::interrupts::idt::{
    InterruptTable,
    HardwareInterrupts
};
use hardware::x86_64::interrupts::handler::{
    InterruptHandler,
    InterruptHandlerWithErrorCode,
    InterruptStackFrameValue
};
use hardware::x86_64::port::Port;
use hardware::x86_64::port::PortReadWrite;
use hardware::x86_64::keyboard::PS2IOPort;
use hardware::x86_64::interrupts::pic;
use multiprocess::executor;
use multiprocess::process::{
    Terminate,
    KeyboardPress,
};
use crate::globals::{
   CHAINED_PICS,
    PROCESS_EXECUTOR,
   VGA_WRITER
};
use alloc::boxed::Box;

// todo: use proper hardware clock
static mut clock : u64 = 0;

pub extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe { send_crash_message_to_current_process();

    writeln!(VGA_WRITER, "Divide by zero occurred in process"); }
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe { writeln!(VGA_WRITER, "BREAKPOINT"); }
}

pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe {send_crash_message_to_current_process();

     writeln!(VGA_WRITER, "Invalid opcode occured in process"); }
}

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: &mut InterruptStackFrameValue, code: u64) {
    unsafe {send_crash_message_to_current_process();

     writeln!(VGA_WRITER, "Page fault occurred in process"); }
}

pub extern "x86-interrupt" fn index_out_of_bounds_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe {send_crash_message_to_current_process();

     writeln!(VGA_WRITER, "Index out of bounds occurred in process"); }
}

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrameValue, error_code : u64) {
    unsafe { writeln!(VGA_WRITER, "DOUBLE FAULT OCCURED"); }
}

pub extern "x86-interrupt" fn keyboard_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe {
        let button_code = PS2IOPort::read_port();

        PROCESS_EXECUTOR.post_message(0, Box::new(KeyboardPress { code : button_code }));

        CHAINED_PICS.notify_end_of_interrupt(HardwareInterrupts::Keyboard as u8);
    }
}

static mut timer_ctr : usize = 0;

pub extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe {

        if timer_ctr > 40 { // emulates tick every 3 secs

            timer_ctr = 0;

            clock += 3000;

            writeln!(VGA_WRITER, "TICK");

            writeln!(VGA_WRITER, "Tick interrupt frame {:?}", stack_frame);

            let interrupted_process_registers = executor::ProcessRegisters {
                instruction_pointer: stack_frame.instruction_pointer,
                stack_pointer: stack_frame.stack_pointer,
                cpu_flags: stack_frame.cpu_flags,
            };

            // update interrupt point for currently running process
            PROCESS_EXECUTOR.save_interrupted_process_return_point(interrupted_process_registers);

            if let Some(next) = PROCESS_EXECUTOR.schedule_next(clock) {
                switch_to_process(next, stack_frame);
            }
        } else {
            timer_ctr += 1;
        }
        CHAINED_PICS.notify_end_of_interrupt(HardwareInterrupts::Timer as u8);
    }
}

/// Switches execution to new process. Process can be an existing one or a completely new one.
/// # Arguments
///  `new_process` - descriptor of the process that will be executed next
///  `stack_frame` - stack frame of the interrupted process
/// # Safety
///// Unsafe because starting new process involves unsafe function `multiprocess::start_new_process`
unsafe fn switch_to_process(new_process : &mut executor::ProcessDescriptor, stack_frame: &mut InterruptStackFrameValue) {
    match new_process.state() {
        executor::ProcessState::Running => {
            switch_registers(new_process.registers(), stack_frame);
        },
        executor::ProcessState::Finished | executor::ProcessState::WaitingForMessage | executor::ProcessState::New => {
            start_new_process(new_process, stack_frame);
        },
        x => {
            panic!("Cannot run a process in state {:?}", x)
        }
    }
}

/// Switches execution to previously stopped process.
/// # Arguments
///  `next_process` - descriptor of the process to switch to
///  `interrupted` - meta data of the stopped process
fn switch_registers(next_process_registers : &executor::ProcessRegisters, interrupted: &mut InterruptStackFrameValue) {
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
fn start_new_process(new_process : &mut executor::ProcessDescriptor, interrupted: &mut InterruptStackFrameValue) {
    let instruction_pointer  = run_process_static as *const () as u64;
    let stack_pointer           = new_process.stack_head_address();

    use hardware::x86_64::registers;
    // the only flag needed, so that interrupts will continue to fire
    // everything else is cleared, because its a new process
    let cpu_flags = registers::interrupt.bits() as u64;

    let new_process_registers = executor::ProcessRegisters {
        instruction_pointer,
        stack_pointer,
        cpu_flags
    };

    // put pointer of the new process descriptor into pocket so `run_process_static` can use it
    // during timer interrupt all other timer interrupts are blocked, so accessing this static var is safe
    unsafe { pocket = new_process as *const _ as u64 };

    // switch registers to point to new process, just like in case of  `executor::ProcessState::Running `
    switch_registers(&new_process_registers, interrupted)
}

unsafe fn send_crash_message_to_current_process() {
    let currently_executing_id = PROCESS_EXECUTOR.currently_executing_id().unwrap();
    let process_description     = PROCESS_EXECUTOR.currently_executing().unwrap().description();

    PROCESS_EXECUTOR.post_message(currently_executing_id, Box::new(Terminate {}));
}

// contains pointer to ProcessDescriptor that is set to execute
static mut pocket : u64 = 0;

fn run_process_static() -> () {
    use core::mem;
    // during timer interrupt all other timer interrupts are blocked, so accessing this static var is safe
    let process_descriptor = unsafe { mem::transmute::<u64, &mut executor::ProcessDescriptor>(pocket) };
    process_descriptor.run_process();
}