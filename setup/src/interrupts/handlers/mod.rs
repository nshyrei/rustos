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
use hardware::x86_64::interrupts::pic;
use multiprocess::executor;
use crate::globals::{
   CHAINED_PICS,
    PROCESS_EXECUTOR
};

use crate::globals::VGA_WRITER;

pub extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe {
        let currently_executing_id =  PROCESS_EXECUTOR.currently_executing();
        let process_description = PROCESS_EXECUTOR.currently_executing_descriptor().description();

        PROCESS_EXECUTOR.remove_currently_executing();

        if let Some(next) = PROCESS_EXECUTOR.schedule_next(0) {
            switch_to_process(next, stack_frame);
        }

        writeln!(VGA_WRITER, "Divide by zero occurred in process with id = {} and description = {}", currently_executing_id, process_description);
    }
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe { writeln!(VGA_WRITER, "BREAKPOINT"); }
}

pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe { writeln!(VGA_WRITER, "INVALID OPCODE OCCURED"); }
}

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: &mut InterruptStackFrameValue, code : u64) {
    unsafe {
        let currently_executing_id =  PROCESS_EXECUTOR.currently_executing();
        let process_description = PROCESS_EXECUTOR.currently_executing_descriptor().description();

        PROCESS_EXECUTOR.remove_currently_executing();

        if let Some(next) = PROCESS_EXECUTOR.schedule_next(0) {
            switch_to_process(next, stack_frame);
        }

        writeln!(VGA_WRITER, "PAGE FAULT occurred in process with id = {} and description = {}", currently_executing_id, process_description);
    }
}

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrameValue, error_code : u64) {
    unsafe { writeln!(VGA_WRITER, "DOUBLE FAULT OCCURED"); }
}

static mut timer_ctr : usize = 0;

static mut clock : u64 = 0;

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

            PROCESS_EXECUTOR.save_interrupted_process_return_point(interrupted_process_registers);

            if let Some(next) = PROCESS_EXECUTOR.schedule_next(clock) {
                switch_to_process(next, stack_frame);
            }
        } else {
            timer_ctr += 1;

            CHAINED_PICS.notify_end_of_interrupt(HardwareInterrupts::Timer as u8);
        }
    }
}

unsafe fn switch_to_process(new_process : &mut executor::ProcessDescriptor, stack_frame: &mut InterruptStackFrameValue) {
    match new_process.state() {
        executor::ProcessState::Running => {
            multiprocess::switch_to_running_process(new_process, stack_frame);

            CHAINED_PICS.notify_end_of_interrupt(HardwareInterrupts::Timer as u8);
        },
        executor::ProcessState::New => {
            hardware::x86_64::interrupts::disable_interrupts();

            hardware::x86_64::interrupts::enable_interrupts();

            CHAINED_PICS.notify_end_of_interrupt(HardwareInterrupts::Timer as u8);

            multiprocess::start_new_process(new_process);
        },
        _ => ()
    }
}