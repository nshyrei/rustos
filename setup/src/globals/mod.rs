use core::ptr;
use core::ops::DerefMut;
use core::fmt::Write;
use core::ops;
use core::cell;
use alloc::boxed::Box;
use pic8259_simple::ChainedPics;

use display::vga::writer::Writer;
use multiprocess::executor;
use hardware::x86_64::interrupts::idt::{
    InterruptTable,
    HardwareInterrupts,
    InterruptTableEntry
};
use hardware::x86_64::interrupts::handler::{
    InterruptHandler,
    InterruptHandlerWithErrorCode
};
use hardware::x86_64::interrupts::{
    pic
};
use x86_64::structures::tss;
use x86_64::structures::gdt;
use hardware::x86_64::keyboard;
use memory::allocator::slab::{
    SlabAllocatorGlobalAlloc,
    SlabAllocator
};
use memory::allocator::bump::ConstSizeBumpAllocator;
use memory::frame::{
    Frame,
    FRAME_SIZE
};
use multiprocess::process::{
   RootProcess,
    HardwareListener,
    SubscribeMe,
    KickStart
};
use crate::process::KeyboardPrinter;

use memory::paging;
use memory::paging::page_table;
use multiboot::multiboot_header::MultibootHeader;
use stdx::macros;
use crate::interrupts::handlers;
use multiprocess::sync::Mutex;

global_fields! {
    VGA_WRITER: Writer = Writer::new();
    PROCESS_EXECUTOR: executor::Executor = executor::Executor::new();
    INTERRUPT_TABLE: InterruptTable = InterruptTable::new();
    CHAINED_PICS: ChainedPics = unsafe { pic::new() } ;
    TASK_STATE_SEGMENT: tss::TaskStateSegment = tss::TaskStateSegment::new();
    GLOBAL_DESCRIPTOR_TABLE : gdt::GlobalDescriptorTable = gdt::GlobalDescriptorTable::new();
}

pub struct Accessor<T> {
    value : cell::Cell<Option<T>>,
}

impl<T> Accessor<T> {
    pub const fn new() -> Self {
        unsafe {
            let value = cell::Cell::new(None);

            Accessor {
                value
            }
        }
    }
}

pub struct CoreProcessesList {
    pub root : u64,

    pub hardware_listener : u64
}

impl CoreProcessesList {
    pub const fn new() -> Self {
        CoreProcessesList {
            root : 0,
            hardware_listener : 0
        }
    }
}

#[global_allocator]
pub static mut HEAP_ALLOCATOR: SlabAllocatorGlobalAlloc = SlabAllocatorGlobalAlloc { value : ptr::NonNull::dangling() };

pub static mut CORE_PROCESSES : CoreProcessesList = CoreProcessesList::new();

pub unsafe fn initialize_keyboard() {

    let result = keyboard::initialize();

    if !result {
        writeln!(VGA_WRITER, "Cannot initialize keyboard, system cannot proceed!");
        panic!();
    }
}

pub unsafe fn create_core_processes() {
    let root_process = RootProcess::new(&mut PROCESS_EXECUTOR);
    let listener = HardwareListener::new(&mut PROCESS_EXECUTOR);

    let root = PROCESS_EXECUTOR.create_process(Box::new(root_process));

    let listener_id = PROCESS_EXECUTOR.fork(root, Box::new(listener));

    let printer = KeyboardPrinter::new(listener_id, &mut PROCESS_EXECUTOR);

    let printer_id = PROCESS_EXECUTOR.fork(root, Box::new(printer));

    CORE_PROCESSES = CoreProcessesList {
        root,
        hardware_listener : listener_id
    };

    PROCESS_EXECUTOR.post_message(printer_id, Box::new(KickStart {}));
}

pub unsafe fn initialize_global_descriptor_table() -> (gdt::SegmentSelector,  gdt::SegmentSelector){
    let code_segment = gdt::Descriptor::kernel_code_segment();
    let tss = gdt::Descriptor::tss_segment(&TASK_STATE_SEGMENT);

    let code_selector = GLOBAL_DESCRIPTOR_TABLE.add_entry(code_segment);
    let tss_selector = GLOBAL_DESCRIPTOR_TABLE.add_entry(tss);

    (code_selector, tss_selector)
}

pub unsafe fn initialize_task_state_segment() {
    use x86_64::VirtAddr;
    let double_fault_stack_address = &double_fault_stack[FRAME_SIZE - 1] as *const _ as u64;
    TASK_STATE_SEGMENT.interrupt_stack_table[0] = VirtAddr::new(double_fault_stack_address);
}

static double_fault_stack : [u8;FRAME_SIZE] = [0 as u8; FRAME_SIZE];

pub unsafe fn load_global_descriptor_table(table : &gdt::GlobalDescriptorTable){
    GLOBAL_DESCRIPTOR_TABLE.load();
}

pub unsafe fn load_task_stack_segment(cs_selector : gdt::SegmentSelector, tss_selector : gdt::SegmentSelector) {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    set_cs(cs_selector);
    load_tss(tss_selector);
}


pub unsafe fn initialize_interrupt_table() {

    // CPU exceptions
    INTERRUPT_TABLE.double_fault = InterruptTableEntry::<InterruptHandlerWithErrorCode>::create_present_entry(handlers::double_fault_handler);
    INTERRUPT_TABLE.page_fault = InterruptTableEntry::<InterruptHandlerWithErrorCode>::create_present_entry(handlers::page_fault_handler);
    let page_fault_options = INTERRUPT_TABLE.page_fault.options_mut();
    page_fault_options.set_stack(0);

    INTERRUPT_TABLE.divide_by_zero = InterruptTableEntry::<InterruptHandler>::create_present_entry(handlers::divide_by_zero_handler);
    INTERRUPT_TABLE.bound_range_exceed = InterruptTableEntry::<InterruptHandler>::create_present_entry(handlers::index_out_of_bounds_handler);
    INTERRUPT_TABLE.invalid_opcode = InterruptTableEntry::<InterruptHandler>::create_present_entry(handlers::invalid_opcode_handler);

    // Hardware interrupts
    INTERRUPT_TABLE.set_interrupt_handler(HardwareInterrupts::Timer as usize, handlers::timer_interrupt_handler);
    INTERRUPT_TABLE.set_interrupt_handler(HardwareInterrupts::Keyboard as usize, handlers::keyboard_handler);

    CHAINED_PICS.initialize();
}

pub fn initialize_memory_allocator(multiboot_header : &MultibootHeader) -> SlabAllocator {

    let (memory_start, memory_end1) = multiboot_header.biggest_memory_area();
    let memory_end = memory_start + 31457280; //30 mb, something bigger than that produces 0x6 crash
    let total_memory = memory_end - memory_start + 1;

    let aux_structures_start_address = preallocate_memory_for_allocator_aux_data_structures(memory_start, memory_end);

    SlabAllocator::new(aux_structures_start_address + 4096, total_memory, memory_end)
}

fn preallocate_memory_for_allocator_aux_data_structures(memory_start : usize, memory_end : usize) -> usize {
    let aux_data_structures_size = SlabAllocator::total_aux_data_structures_size(memory_start, memory_end);

    let premade_bump_end_address  = Frame::address_align_up(memory_start + aux_data_structures_size);
    let mut premade_bump= ConstSizeBumpAllocator::from_address(memory_start, premade_bump_end_address, FRAME_SIZE);

    // memory layout for allocator
    // |***********************|****************************|****************|
    // |aux structures page tables|aux structures working memory|user memory pool|

    // premap memory for memory allocator inner data structures
    let aux_structures_start_address = premade_bump_end_address + FRAME_SIZE; // next frame
    let aux_structures_end_address = Frame::address_align_up(aux_structures_start_address + aux_data_structures_size);

    // map all pages in range 1 to 1
    for frame in Frame::range_inclusive(aux_structures_start_address, aux_structures_end_address) {
        let p4_table = paging::p4_table();
        p4_table.map_page_1_to_1(frame, page_table::PRESENT | page_table::WRITABLE, &mut premade_bump);
    }

    // perform simple presence test
    test_allocator_aux_data_structures_memory(aux_structures_start_address, aux_structures_end_address);

    aux_structures_start_address
}

fn test_allocator_aux_data_structures_memory(aux_structures_start_address : usize, aux_structures_end_address : usize) {
    for frame in Frame::range_inclusive(aux_structures_start_address, aux_structures_end_address) {
        let p4_table = paging::p4_table();
        let present = p4_table.is_present(frame);

        unsafe { writeln!(VGA_WRITER, "Is present {}, val {}", frame, present); }

        Frame::zero_frame(&frame);
    }
}