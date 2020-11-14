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
            else {
                CHAINED_PICS.notify_end_of_interrupt(HardwareInterrupts::Timer as u8);
            }
        } else {
            timer_ctr += 1;

            CHAINED_PICS.notify_end_of_interrupt(HardwareInterrupts::Timer as u8);
        }
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
            multiprocess::switch_to_running_process(new_process, stack_frame);

            CHAINED_PICS.notify_end_of_interrupt(HardwareInterrupts::Timer as u8);
        },
        executor::ProcessState::Finished | executor::ProcessState::WaitingForMessage => {
            hardware::x86_64::interrupts::disable_interrupts();

            hardware::x86_64::interrupts::enable_interrupts();

            CHAINED_PICS.notify_end_of_interrupt(HardwareInterrupts::Timer as u8);

            new_process.run_process();
        },
        executor::ProcessState::New => {
            // todo find out why this disable/enable is needed or otherwise the interrupt wont fire again
            hardware::x86_64::interrupts::disable_interrupts();

            hardware::x86_64::interrupts::enable_interrupts();

            CHAINED_PICS.notify_end_of_interrupt(HardwareInterrupts::Timer as u8);

            /*use multiprocess::executor::run_process_static;
            use core::mem;

            let adr = run_process_static as *const () as u64;
            stack_frame.instruction_pointer = adr;

            let stack_address           = new_process.stack_address() as u32;
            let descriptor_address   = new_process as *const _ as u64;

            // push process descriptor pointer into new process stack
            core::ptr::write_unaligned((stack_address - (mem::size_of::<u64>() as u32)) as *mut u64, descriptor_address);

            let next_process_registers = new_process.registers();
            stack_frame.stack_pointer           = new_process.stack_address();
            stack_frame.cpu_flags                 = next_process_registers.cpu_flags;*/

            multiprocess::start_new_process(new_process);
        },

        executor::ProcessState::Crashed => {
            let s = 10;
            let ss = s;
        },

        executor::ProcessState::AskedToTerminate => {
            let s = 10;
            let ss = s;
        },
        executor::ProcessState::WaitingForResource => {
            let s = 10;
            let ss = s;
        }
    }
}

unsafe fn send_crash_message_to_current_process() {
    let currently_executing_id = PROCESS_EXECUTOR.currently_executing_id().unwrap();
    let process_description     = PROCESS_EXECUTOR.currently_executing().unwrap().description();

    PROCESS_EXECUTOR.post_message(currently_executing_id, Box::new(Terminate {}));
}