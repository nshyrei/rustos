use ::x86_64::interrupts::handler::{InterruptHandler, InterruptHandlerWithErrorCode};
use ::x86_64::interrupts::pic::PIC_1_OFFSET;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum HardwareInterrupts {
    Timer = PIC_1_OFFSET,
}

/// Describes entry of interrupt descriptor table (IDT).
#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptTableEntry<HandlerFunc> {
    // first 16 bits of interrupt handler address
    lower_pointer_bits : u16,
    // the GDT selector value
    gdt_selector : GDTSelector,
    // interrupt handler options
    options : InterruptOptions,
    // bits 16-32 (excl) of interrupt handler address
    middle_pointer_bits : u16,
    // high 32 bits of interrupt handler address
    remaining_pointer_bits : u32,
    // reserved field according to processor spec
    reserved : u32,
    ph : PhantomData<HandlerFunc>
}

impl<HandlerFunc> InterruptTableEntry<HandlerFunc> {
    /// Creates minimal working table entry. The newly created entry is not visible for interrupt controller,
    /// to make it visible you need to change visibility in `options` field.
    /// # Arguments
    /// `handler_address` - address of interrupt handling function
    fn new(handler_address : u64) -> Self {

        let lower_pointer_bits              = handler_address as u16;
        let middle_pointer_bits            = (handler_address >> 16) as u16;
        let remaining_pointer_bits      = (handler_address >> 32) as u32;
        let options           = InterruptOptions::new_present();
        let gdt_selector = GDTSelector::new();

        InterruptTableEntry {
            lower_pointer_bits,
            gdt_selector,
            options,
            middle_pointer_bits,
            remaining_pointer_bits,
            reserved : 0,
            ph : PhantomData
        }
    }



    /// Creates empty table entry.
    /// This entry is not visible to controller and doesnt point to valid handler function, it is used only for initial table initialization.
    const fn empty() -> Self {

        let options = MINIMAL_INTERRUPT_OPTIONS;
        let gdt_selector = GDTSelector::empty();

        InterruptTableEntry {
            lower_pointer_bits : 0,
            gdt_selector,
            options,
            middle_pointer_bits : 0,
            remaining_pointer_bits : 0,
            reserved : 0,
            ph : PhantomData
        }
    }
}

impl InterruptTableEntry<InterruptHandler> {
    pub fn create_present_entry(handler : InterruptHandler) -> Self {
        let mut result = InterruptTableEntry::<InterruptHandler>::new(handler as u64);
        result.options.set_present();

        result
    }
}

impl InterruptTableEntry<InterruptHandlerWithErrorCode> {
    pub fn create_present_entry(handler : InterruptHandlerWithErrorCode) -> Self {
        let mut result = InterruptTableEntry::<InterruptHandlerWithErrorCode>::new( handler as u64);
        result.options.set_present();

        result
    }
}

/// Interrupt table, contains entries describing how processor handles  interrupt signals.
#[repr(C)]
#[repr(align(16))]
pub struct InterruptTable {

    // 32 handlers for cpu exceptions
    pub divide_by_zero : InterruptTableEntry<InterruptHandler>,

    pub debug : InterruptTableEntry<InterruptHandler>,

    pub non_maskable_interrupt : InterruptTableEntry<InterruptHandler>,

    pub breakpoint : InterruptTableEntry<InterruptHandler>,

    pub overflow : InterruptTableEntry<InterruptHandler>,

    pub bound_range_exceed : InterruptTableEntry<InterruptHandler>,

    pub invalid_opcode : InterruptTableEntry<InterruptHandler>,

    pub device_not_available : InterruptTableEntry<InterruptHandler>,

    pub double_fault : InterruptTableEntry<InterruptHandlerWithErrorCode>,

    coprocessor_segment_overrun : InterruptTableEntry<InterruptHandler>,

    pub invalid_tss : InterruptTableEntry<InterruptHandler>,

    pub segment_not_present : InterruptTableEntry<InterruptHandler>,

    pub stack_segment_fault : InterruptTableEntry<InterruptHandler>,

    pub general_protection_fault : InterruptTableEntry<InterruptHandler>,

    pub page_fault : InterruptTableEntry<InterruptHandlerWithErrorCode>,

    reserved_0 : InterruptTableEntry<InterruptHandler>,

    pub x87_floating_point_exception : InterruptTableEntry<InterruptHandler>,

    pub aligment_check : InterruptTableEntry<InterruptHandler>,

    pub machine_check : InterruptTableEntry<InterruptHandler>,

    pub simd_floating_point_exception : InterruptTableEntry<InterruptHandler>,

    pub virtualization_exception : InterruptTableEntry<InterruptHandler>,

    reserved_1 : [InterruptTableEntry<InterruptHandler>; 9],

    pub security_exception : InterruptTableEntry<InterruptHandler>,

    reserved_10 : InterruptTableEntry<InterruptHandler>,

    // handlers for user defined and hardware interrupts
    interrupts : [InterruptTableEntry<InterruptHandler>; 256 - 32]
}

impl InterruptTable {
    
    /// Creates new table filed with empty entries.
    pub const fn new() -> Self {
        InterruptTable {
            divide_by_zero: InterruptTableEntry::empty(),
            debug: InterruptTableEntry::empty(),
            non_maskable_interrupt: InterruptTableEntry::empty(),
            breakpoint: InterruptTableEntry::empty(),
            overflow: InterruptTableEntry::empty(),
            bound_range_exceed: InterruptTableEntry::empty(),
            invalid_opcode: InterruptTableEntry::empty(),
            device_not_available: InterruptTableEntry::empty(),
            double_fault: InterruptTableEntry::empty(),
            coprocessor_segment_overrun: InterruptTableEntry::empty(),
            invalid_tss: InterruptTableEntry::empty(),
            segment_not_present: InterruptTableEntry::empty(),
            stack_segment_fault: InterruptTableEntry::empty(),
            general_protection_fault: InterruptTableEntry::empty(),
            page_fault: InterruptTableEntry::empty(),
            reserved_0: InterruptTableEntry::empty(),
            x87_floating_point_exception: InterruptTableEntry::empty(),
            aligment_check: InterruptTableEntry::empty(),
            machine_check: InterruptTableEntry::empty(),
            simd_floating_point_exception: InterruptTableEntry::empty(),
            virtualization_exception: InterruptTableEntry::empty(),
            reserved_1: [InterruptTableEntry::empty(); 9],
            security_exception: InterruptTableEntry::empty(),
            reserved_10: InterruptTableEntry::empty(),
            interrupts :  [InterruptTableEntry::empty(); 256 - 32]
        }
    }

    /// Creates entry for interrupt handler denoted by idx.
    /// # Arguments
    /// `idx` - handler index
    /// `handler` - interrupt handler function
    /// # Panic
    ///  Panics if `idx` is out of range or points to reserved entry.
    pub fn set_interrupt_handler(&mut self, idx : usize, handler : InterruptHandler) {
        let entry = InterruptTableEntry::<InterruptHandler>::create_present_entry(handler);

        self[idx] = entry
    }

    /// Creates a pointer for this table. Used only for `load_table` function.
    pub(crate) fn pointer(&self) -> InterruptTablePointer {
        use core::mem;

        let base = self as *const _ as u64;
        let limit = (mem::size_of::<Self>() - 1) as u16; // -1 because address must be inclusive according to spec

        InterruptTablePointer {
            limit,
            base
        }
    }
}

impl Index<usize> for InterruptTable {
    type Output = InterruptTableEntry<InterruptHandler>;

    fn index(&self, index: usize) -> &InterruptTableEntry<InterruptHandler> {
        match index {
            i @ 32 ..=255 => &self.interrupts[i - 32],
            _ => panic!("Interrupt table index out of range")
        }
    }
}

impl IndexMut<usize> for InterruptTable {
    fn index_mut(&mut self, index: usize) -> &mut InterruptTableEntry<InterruptHandler> {
        match index {
            i @ 32 ..=255 => &mut self.interrupts[i - 32],
            _ => panic!("Interrupt table index out of range")
        }
    }
}

/// Describes interrupt entry options.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptOptions {
    value : u16
}

/// A minimal valid options record.
const MINIMAL_INTERRUPT_OPTIONS : InterruptOptions = InterruptOptions {
    value : 0b1110_0000_0000
};

impl InterruptOptions {

    /// Creates minimal options record and sets it to present.
    pub fn new_present() -> Self {
        let mut minimal = MINIMAL_INTERRUPT_OPTIONS;

        minimal.set_present();

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

    /// Sets this interrupt handler as present.
    pub fn set_present(&mut self) {
        let mut flags = self.flags();

        flags.insert(IS_PRESENT);

        self.value = flags.bits();
    }

    /// Sets this entry as hidden. No interrupts will get handled for that handler.
    pub fn set_unused(&mut self) {
        let mut flags = self.flags();
        flags.remove(IS_PRESENT);

        self.value = flags.bits();
    }
}

bitflags! {
    pub struct InterruptOptionsFlags : u16 {
        const DISABLE_INTERRUPT = 1 << 8;
        const ALWAYS_PRESENT =      1 << 9;
        const ALWAYS_PRESENT1 =    1 << 10;
        const ALWAYS_PRESENT2 =    1 << 11;
        const IS_PRESENT =                 1 << 15;
    }
}

/// Describes a pointer to descriptor table.
/// Used only for `load_interrupt_table` function
#[repr(C, packed)]
pub(crate) struct InterruptTablePointer {
    limit : u16,
    base : u64
}

/// Describes segment selector for descriptor table.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct GDTSelector {
    value : u16
}

impl GDTSelector {
    /// Creates new valid segment selector.
    pub fn new() -> Self {
        use x86_64::registers;

        let cs_value = registers::cs();

        GDTSelector {
            value : cs_value
        }
    }

    /// Empty selector pointing to invalid memory area. Used only for table initialization
    const fn empty() -> Self {
        GDTSelector {
            value : 0
        }
    }
}