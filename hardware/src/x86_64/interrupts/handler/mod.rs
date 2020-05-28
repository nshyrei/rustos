pub type InterruptHandler                               = extern "x86-interrupt" fn (&mut InterruptStackFrameValue);

pub type InterruptHandlerWithErrorCode  = extern "x86-interrupt" fn (&mut InterruptStackFrameValue, u64);

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    pub instruction_pointer: u64,
    /// The code segment selector, padded with zeros.
    pub code_segment: u64,
    /// The flags register before the interrupt handler was invoked.
    pub cpu_flags: u64,
    /// The stack pointer at the time of the interrupt.
    pub stack_pointer: u64,
    /// The stack segment descriptor at the time of the interrupt (often zero in 64-bit mode).
    pub stack_segment: u64,
}