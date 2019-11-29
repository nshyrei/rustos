pub type InterruptHandler = extern "x86-interrupt" fn (&InterruptStackFrameValue);

#[derive(Clone)]
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptTableEntry {
    lower_pointer_bits : u16,
    gdt_selector : GDTSelector,
    options : InterruptOptions,
    middle_pointer_bits : u16,
    remaining_pointer_bits : u32,
    reserved : u32
}

impl InterruptTableEntry {
    pub fn new(gdt_selector : GDTSelector, handler : InterruptHandler) -> Self {
        let handler_address = handler as u64;

        let lower_pointer_bits              = handler_address as u16;
        let middle_pointer_bits             = (handler_address >> 16) as u16;
        let remaining_pointer_bits      = (handler_address >> 32) as u32;
        let options                                     = InterruptOptions::new();

        InterruptTableEntry {
            lower_pointer_bits,
            gdt_selector,
            options,
            middle_pointer_bits,
            remaining_pointer_bits,
            reserved : 0
        }
    }

    pub fn empty() -> Self {

        let options             = InterruptOptions::new();
        let gdt_selector    = GDTSelector::new(0, 0);

        InterruptTableEntry {
            lower_pointer_bits : 0,
            gdt_selector,
            options,
            middle_pointer_bits : 0,
            remaining_pointer_bits : 0,
            reserved : 0
        }
    }
}

use core::ptr;

pub struct InterruptTableHelp {
    pub value : Option<ptr::NonNull<InterruptTable>>
}

#[repr(C)]
pub struct InterruptTable {
    // handlers for cpu exceptions
    cpu_exceptions: [InterruptTableEntry; 32],
    // handlers for user defined interrupts
    interrupts : [InterruptTableEntry; 256 - 32]
}

impl InterruptTable {
    pub fn new() -> Self {
        let cpu_exceptions = [InterruptTableEntry::empty(); 32];
        let interrupts = [InterruptTableEntry::empty(); 256 - 32];

        InterruptTable {
            cpu_exceptions,
            interrupts
        }
    }

    pub fn set_handler(&mut self, index : usize, handler : InterruptHandler) {
        let mut entry = InterruptTableEntry::new(GDTSelector::new(0, 0), handler);
        entry.options.set_present();

        self.set_entry(index, entry)
    }

    pub fn set_entry(&mut self, index : usize, entry : InterruptTableEntry) {
        self.cpu_exceptions[index] = entry
    }

    fn pointer(&self) -> InterruptTablePointer {
        use core::mem;

        let base = self as *const _ as u64;
        let limit = (mem::size_of::<Self>() - 1) as u16; // -1 because address must be inclusive according to spec

        InterruptTablePointer {
            limit,
            base
        }
    }
}

pub unsafe fn load_interrupt_table(table : &InterruptTable){
    let ptr = &table.pointer();

    asm!("lidt ($0)" :: "r" (ptr) : "memory")
}

#[repr(C, packed)]
struct InterruptTablePointer {
    limit : u16,
    base : u64
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GDTSelector {
    value : u16
}

impl GDTSelector {
    pub fn new(index : u16, privilege_level : u16) -> Self {
        use x86_64::registers;

        let cs_value = registers::cs();

        //let new_value = index << 3 | privilege_level;

        GDTSelector {
            value : cs_value
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptOptions {
    value : u16
}

impl InterruptOptions {

    pub fn minimal() -> Self {
        let validValue = 0b1110_0000_0000; // bits 9-11 must be set to 1 according to spec

        InterruptOptions {
            value : validValue
        }
    }

    pub fn new() -> Self {
        let mut minimal = InterruptOptions::minimal();

        minimal.set_present();
        minimal.disable_interrupts();

        minimal
    }

    pub fn value(&self) -> u16 {
        self.value
    }

    pub fn flags(&self) -> InterruptOptionsFlags {
        InterruptOptionsFlags::from_bits_truncate(self.value)
    }

    pub fn set_flags(&mut self, new_flags : InterruptOptionsFlags) {
        self.value = new_flags.bits();
    }

    pub fn disable_interrupts(&mut self) {
        let mut flags = self.flags();

        flags.remove(DISABLE_INTERRUPTS);

        self.value = flags.bits();
    }

    pub fn set_present(&mut self) {
        let mut flags = self.flags();

        flags.insert(IS_PRESENT);

        self.value = flags.bits();
    }

    pub fn set_unused(&mut self) {
        let mut flags = self.flags();
        flags.remove(IS_PRESENT);

        self.value = flags.bits();
    }
}

bitflags! {
    pub struct InterruptOptionsFlags : u16 {
        const DISABLE_INTERRUPTS = 1 << 8;
        const ALWAYS_PRESENT = 1 << 9;
        const ALWAYS_PRESENT1 = 1 << 10;
        const ALWAYS_PRESENT2 = 1 << 11;
        const IS_PRESENT =      1 << 15;

    }
}