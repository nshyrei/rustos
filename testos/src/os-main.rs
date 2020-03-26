#![feature(lang_items)]
#![feature(asm)]
#![no_std]
#![feature(abi_x86_interrupt)]

extern crate rlibc;
extern crate multiboot;
extern crate display;
extern crate memory;
extern crate hardware;
extern crate alloc;
extern crate malloc;
extern crate stdx_memory;
extern crate stdx;
extern crate pic8259_simple;
extern crate multiprocess;

use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags::{basic_memory_info, elf, memory_map};
use multiboot::multiboot_header::tags::memory_map::*;
use display::vga::writer::Writer;
use memory::allocator::bump::BumpAllocator;
use memory::allocator::bump::ConstSizeBumpAllocator;
use memory::frame::frame_allocator::*;
use memory::frame::Frame;
use memory::frame::FRAME_SIZE;
use memory::paging;
use memory::paging::page_table;
use memory::paging::page_table::P4Table;
use stdx_memory::MemoryAllocator;
use stdx_memory::MemoryAllocatorMeta;
use core::clone::Clone;
use core::fmt::Write;
use alloc::boxed::Box;
use alloc::vec::Vec;
use malloc::TestAllocator;
use stdx_memory::collections::immutable::double_linked_list::DoubleLinkedList;
use memory::allocator::slab::SlabAllocator;
use memory::allocator::slab::SlabHelp;
use memory::allocator::buddy::BuddyAllocator;

use hardware::x86_64::registers;
use hardware::x86_64::interrupts;
use hardware::x86_64::interrupts::{InterruptTableHelp, InterruptTable, HardwareInterrupts, CPUInterrupts, InterruptStackFrameValue};
use core::ptr;
use core::ops::DerefMut;
use core::cell;
use alloc::alloc::Layout;
use alloc::rc::Rc;
use stdx_memory::heap;
use multiprocess::{Process, Message};
use multiprocess::executor;
use multiprocess::process;
use pic8259_simple::ChainedPics;

static mut VGA_WRITER: Option<Writer> = None;

#[global_allocator]
static mut HEAP_ALLOCATOR: SlabHelp = SlabHelp { value : ptr::NonNull::dangling() };

static mut interruptTable : InterruptTable = InterruptTable::new();

static mut chained_pics : ChainedPics = unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) } ;

static mut process_executor : executor::ExecutorHelp = executor::ExecutorHelp { value : ptr::NonNull::dangling() };

static mut DummyAddr : u64 = 0;

#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "C" fn rust_main(multiboot_header_address: usize) {
    unsafe {

        let multiboot_header = MultibootHeader::load(multiboot_header_address);

        VGA_WRITER = Some(Writer::new());

        //print_multiboot_data(multiboot_header, VGA_WRITERG.as_mut().unwrap());

        let mut frame_allocator = FrameAllocator::new(multiboot_header);

        paging::remap_kernel(&mut paging::p4_table(), &mut frame_allocator, multiboot_header);

       let (memory_start, memory_end1) = multiboot_header.biggest_memory_area();
        let memory_end = memory_start + 31457280; //30 mb, something bigger than that produces 0x6 crash
        let total_memory = memory_end - memory_start + 1;

        let aux_structures_start_address = preallocate_memory_for_allocator_aux_data_structures(memory_start, memory_end);

        let mut slab_allocator = SlabAllocator::new2(aux_structures_start_address + 4096, total_memory, memory_end, VGA_WRITER.as_mut().unwrap());

        HEAP_ALLOCATOR.value = ptr::NonNull::new_unchecked(&mut slab_allocator as *mut SlabAllocator);

        memory_allocator_should_properly_allocate_and_free_memory();

        interruptTable.set_cpu_interrupt_handler_with_error_code(CPUInterrupts::DoubleFault, double_fault_handler2);
        interruptTable.set_cpu_interrupt_handler_with_error_code(CPUInterrupts::PageFault, page_fault_handler);
       // interruptTable.set_cpu_interrupt_handler(CPUInterrupts::DivideByZero, div_handler);

        //interruptTable.set_cpu_interrupt_handler(CPUInterrupts::InvalidOpcode, breakpoint_handler1);
        interruptTable.set_hardware_interrupt_handler(HardwareInterrupts::Timer, timer_interrupt_handler);
        interrupts::load_interrupt_table(&interruptTable);

        chained_pics.initialize();

        let mut executor = Rc::new(cell::UnsafeCell::new(executor::Executor::new()));

        process_executor.value =  ptr::NonNull::new_unchecked(&mut executor as *mut executor::ExecutorRef);

        use core::mem;
        use core::ops::Deref;

        let dummy_process = DummyProcess { value : 1000 };
        let dummy_process_state_box = Box::new(dummy_process);
        DummyAddr = DummyProcess::process as *const fn() as u64;
       /* let dummy_process_ptr = DummyProcess::process;
        let dummy_process_box = mem::transmute::<fn(&mut DummyProcess), fn(&mut Process)>(dummy_process_ptr);
        let dummy_addr = dummy_process_box as u64;

        use hardware::x86_64::registers;
        let addr = dummy_process_state_box.deref() as *const _ as u64;
        registers::pushq(addr);
        registers::jump(dummy_addr);
*/
        process_executor.create_process( dummy_process_state_box);
        process_executor.post_message(0, Box::new(IncreaseCtr {}));

        /*let mut root_process = process::RootProcess::new(Rc::clone(&executor));

        let sample_process = SampleProcess {
            root :  process::ProcessRef::clone(&root_process),

            child : None,

            ctr : 0
        };

        let sample_process_box = Box::new(sample_process);

        let mut sample_ref = root_process.fork(sample_process_box);

        {
            sample_ref.post_message(Box::new(process::StartProcess {} ));
        }

        let sender_process = SenderProcess {
            root : process::ProcessRef::clone(&root_process),

            child : sample_ref,
        };

        let sender_process_box = Box::new(sender_process);

        {
            root_process.fork(sender_process_box);
        }*/

        interruptTable.enable_hardware_interrupts();

        //asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel");
        //writeln!(&mut vga_writer, "{}", predefined_p4_table);

        // run pre-init tests
        let p4_table = paging::p4_table();

        /*paging_map_should_properly_map_pages(p4_table, slab_allocator.frame_allocator(), &mut vga_writer);
        paging_translate_page_should_properly_translate_pages(p4_table, slab_allocator.frame_allocator());
        paging_unmap_should_properly_unmap_elements(p4_table, slab_allocator.frame_allocator());
        paging_translate_address_should_properly_translate_virtual_address(p4_table, slab_allocator.frame_allocator());*/
    }
    loop {
        unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "Main thread end loop!"); };
    }
}

#[repr(C)]
pub struct DummyProcess {

    value : usize
}

impl DummyProcess {

    fn process() -> () {
        unsafe {

            writeln!(VGA_WRITER.as_mut().unwrap(), "I am dummy!, with value ");

            //loop {}
        }
    }

    fn process_message(message: Message) -> () {
        unsafe {
            writeln!(VGA_WRITER.as_mut().unwrap(), "I am dummy!");

            loop {}
        }
    }
}


impl Process for DummyProcess {
    fn process_message(&mut self, message: Message) -> () {
        unsafe {
            writeln!(VGA_WRITER.as_mut().unwrap(), "I am dummy!");

            loop {}
        }
    }
}

pub struct IncreaseCtr {}


pub struct SenderProcess {

    pub root : process::ProcessRef,

    pub child : process::ProcessRef,
}

impl Process for SenderProcess {
    fn process_message(&mut self, message: Message) -> () {
        unsafe {
            writeln!(VGA_WRITER.as_mut().unwrap(), "Sending inc to Id = {}!", self.child.id());

            self.child.post_message(Box::new(IncreaseCtr {}));
        }
    }
}

/*pub struct SampleProcess {

    pub root : process::ProcessRef,

    pub child : Option<process::ProcessRef>,

    pub ctr : usize
}

impl Process for SampleProcess {
    fn process_message(&mut self, message: Message) -> () {
        if message.is::<process::StartProcess>() {

            unsafe {
                writeln!(VGA_WRITER.as_mut().unwrap(), "Thats funny I am creating a process Id = {}!", 1);
            }
        }

        if message.is::<IncreaseCtr>() {

            unsafe {

                if (self.ctr > 3) {

                    writeln!(VGA_WRITER.as_mut().unwrap(), "Thats funny I am creating a new child!");

                    let sample_process = SampleProcess {
                        root :  process::ProcessRef::clone(&self.root),

                        child : None,

                        ctr : 0
                    };

                    let sample_process_box = Box::new(sample_process);

                    let mut child_ref = self.root.fork(sample_process_box);

                    child_ref.post_message(Box::new(process::StartProcess {} ));

                    self.child = Some(child_ref);
                    self.ctr = 0;
                }
                else {

                    writeln!(VGA_WRITER.as_mut().unwrap(), "Got inc ctr!");

                    self.ctr += 1;
                }
            }
        }
    }
}*/

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

static mut timer_ctr : usize = 0;

fn switch_to_running_process(new_process : &executor::ProcessDescriptor, interrupted: &mut InterruptStackFrameValue) {
    let new_process_registers = new_process.registers();

    interrupted.instruction_pointer = new_process_registers.instruction_pointer;
    interrupted.stack_pointer           = new_process_registers.stack_pointer;
    interrupted.cpu_flags                 = new_process_registers.cpu_flags;
}

fn switch_to_new_process(new_process : &mut executor::ProcessDescriptor) {
    if let Some(mail) = new_process.mailbox_mut().pop_front() {
        let process_func = new_process.process();

        process_func.process_message(mail);
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe {

        if timer_ctr > 40 { // emulates tick every 3 secs

            timer_ctr = 0;

            writeln!(VGA_WRITER.as_mut().unwrap(), "TICK");

            writeln!(VGA_WRITER.as_mut().unwrap(), "Tick interrupt frame {:?}", stack_frame);

            let interrupted_process_registers = executor::ProcessRegisters {
                instruction_pointer: stack_frame.instruction_pointer,
                stack_pointer: stack_frame.stack_pointer,
                cpu_flags: stack_frame.cpu_flags,
            };

            process_executor.post_message(2, Box::new(IncreaseCtr {}));

            process_executor.update_current_process(interrupted_process_registers);

            if let Some(next) = process_executor.schedule_next() {

                 match next.state() {
                     executor::ProcessState::Running => {

                         switch_to_running_process(next, stack_frame);

                         chained_pics.notify_end_of_interrupt(InterruptIndex::Timer as u8);
                     },
                     executor::ProcessState::New => {
                         hardware::x86_64::interrupts::disable_interrupts();

                         hardware::x86_64::interrupts::enable_interrupts();

                         chained_pics.notify_end_of_interrupt(InterruptIndex::Timer as u8);

                         switch_to_new_process(next);
                     },
                     _ => ()
                 }
            }
        } else {
            timer_ctr += 1;

            chained_pics.notify_end_of_interrupt(InterruptIndex::Timer as u8);
        }
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "BREAK NIIGGA"); }
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: &mut InterruptStackFrameValue) {
    unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "OPCODE NIIGGA"); }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: &mut InterruptStackFrameValue, code : u64) {
    unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "PAGE NIIGGA"); }
}

extern "x86-interrupt" fn double_fault_handler2(stack_frame: &mut InterruptStackFrameValue, error_code : u64) {
    unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "DOUBLE NIIGGA"); }
}

fn memory_allocator_should_properly_allocate_and_free_memory() {
    // everything inside inner block will get deleted after block exit
    {
        let mut test_array : [u64;6] = [0 ;6];

        {
            let boxin = Box::new(test_array);

            let b = boxin;
        }
    }

    let result = unsafe { HEAP_ALLOCATOR.is_fully_free() };

    assert_eq!(result, true, "Allocator wasn't fully free after allocating memory in isolated block");
}

fn preallocate_memory_for_allocator_aux_data_structures(memory_start : usize, memory_end : usize) -> usize {
    let aux_data_structures_size = SlabAllocator::total_aux_data_structures_size(memory_start, memory_end);

    let premade_bump_end_address  = Frame::address_align_up(memory_start + aux_data_structures_size);
    let mut premade_bump                = ConstSizeBumpAllocator::from_address(memory_start, premade_bump_end_address, FRAME_SIZE);

    // |aux structures page tables|aux structures working memory|allocator working memory|
    // premap memory for memory allocator inner data structures
    let aux_structures_start_address = premade_bump_end_address + FRAME_SIZE; // next frame
    let aux_structures_end_address = Frame::address_align_up(aux_structures_start_address + aux_data_structures_size);

    for frame in Frame::range_inclusive(aux_structures_start_address, aux_structures_end_address) {
        let p4_table = paging::p4_table();
        p4_table.map_page_1_to_1(frame, page_table::PRESENT | page_table::WRITABLE, &mut premade_bump);
    }

    test_allocator_aux_data_structures_memory(aux_structures_start_address, aux_structures_end_address);

    aux_structures_start_address
}

fn test_allocator_aux_data_structures_memory(aux_structures_start_address : usize, aux_structures_end_address : usize) {
    for frame in Frame::range_inclusive(aux_structures_start_address, aux_structures_end_address) {
        let p4_table = paging::p4_table();
        let present = p4_table.is_present(frame);

        unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "Is present {}, val {}", frame, present); }

        Frame::zero_frame(&frame);
    }
}

pub fn slab_allocator_should_be_fully_free(writer: &'static mut Writer, adr: usize) -> SlabAllocator {
    use memory::frame::Frame;
    use memory::frame::FRAME_SIZE;
    use stdx::iterator::IteratorExt;
    use stdx::Sequence;
    use stdx::Iterable;
    use stdx_memory::MemoryAllocator;
    use stdx_memory::collections::double_linked_list::{DoubleLinkedList, DoubleLinkedListIterator, BuddyMap};
    use memory::allocator::bump::BumpAllocator;
    use memory::allocator::buddy::BuddyAllocator;
    use memory::allocator::free_list::FreeListAllocator;
    use core::mem;
    use core::ptr;
    use stdx_memory::heap;

    let size = 32768;
    let heap: [u8; 80000] = [0; 80000];
    let heap_addr = heap.as_ptr() as usize;

    let memory_start = Frame::address_align_up(heap_addr);

    let memory_end = memory_start + 40000;
    let aux_data_structures_size = SlabAllocator::total_aux_data_structures_size(memory_start, memory_end);

    let mut slab_allocator = SlabAllocator::new(memory_start, memory_end, writer);

   let result = slab_allocator.allocate(2048);
    let result1 = slab_allocator.allocate(2048);
    let result2 = slab_allocator.allocate(2048);

    slab_allocator.free(result1.unwrap());
    slab_allocator.free(result.unwrap());
    slab_allocator.free(result2.unwrap());

    let result = slab_allocator.is_fully_free();

    slab_allocator
}

use core::panic::PanicInfo;

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
#[lang = "panic_impl"]
#[no_mangle]
pub extern "C" fn panic_impl(pi: &PanicInfo) -> ! {

    //unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "Rust code panicked with {}", pi.message()); }

    loop {}
}
#[lang = "oom"]
#[no_mangle]
pub extern "C" fn oom(l: Layout) -> ! {
    loop {}
}

fn print_multiboot_data(multiboot_header : &MultibootHeader, vga_writer : &mut Writer) {
    writeln!(vga_writer, "---Basic memory info---");

    let mem_info = multiboot_header
            .read_tag::<memory_map::MemoryMap>()
            .unwrap();

    let mut mem_sections =
            mem_info
                .entries()
                .filter(|e| e.entry_type() == memory_map::MemoryMapEntryType::Available as u32);

    writeln!(vga_writer, "---Available memory {}", mem_info.available_memory());
    writeln!(vga_writer, "---Memory sections---");
    while let Some(e) = mem_sections.next() {
        writeln!(vga_writer, "{}", e);
    }

    let elf_sections = multiboot_header
            .read_tag::<elf::ElfSections>()
            .unwrap();
    let mut elf_sections_it = elf_sections.entries();

    writeln!(vga_writer, "---Elf sections---");
    writeln!(vga_writer, "Elf sections start: {}", elf_sections.entries_start_address().unwrap());
    writeln!(vga_writer, "Elf sections end: {}", elf_sections.entries_end_address().unwrap());

    while let Some(e) = elf_sections_it.next() {
        writeln!(vga_writer, "{}", e);
    }
}

unsafe fn paging_map_should_properly_map_pages(page_table : &mut page_table::P4Table, frame_alloc : &mut BuddyAllocator, vga_writer : &mut Writer) {

    let virtual_frame = Frame::from_address(0x400000000000);
    let physical_frame = Frame::from_address(frame_alloc.allocate(FRAME_SIZE).expect("No frames for paging test"));

    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT | page_table::WRITABLE, frame_alloc);

    // try read whole frame from virtual address, if it succeeds without a segfault then
    // map function worked correctly
    let virtual_frame_address = virtual_frame.address();

    for i in 0..FRAME_SIZE {
        unsafe {
            // reading into var is important to prevent compiler optimizing the read away
            let _result = *((virtual_frame_address + i as usize) as *const u8);
        }
    }

    frame_alloc.free(physical_frame.address());
    page_table.unmap_page(virtual_frame);
}

unsafe fn paging_translate_page_should_properly_translate_pages(page_table : &mut page_table::P4Table, frame_alloc : &mut BuddyAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = Frame::from_address(frame_alloc.allocate(FRAME_SIZE).expect("No frames for paging test"));

    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT, frame_alloc);

    let result = page_table.translate_page(virtual_frame);

    sanity_assert_translate_page_result(virtual_frame, physical_frame, result);

    frame_alloc.free(physical_frame.address());
    page_table.unmap_page(virtual_frame);
}

unsafe fn paging_translate_address_should_properly_translate_virtual_address(page_table : &mut page_table::P4Table, frame_alloc : &mut BuddyAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = Frame::from_address(frame_alloc.allocate(FRAME_SIZE).expect("No frames for paging test"));

    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT, frame_alloc);

    let virtual_frame_address = virtual_frame.address();
    let physical_frame_address = physical_frame.address();

    for frame_offset in 0..FRAME_SIZE {
        let virtual_address  = virtual_frame_address + frame_offset as usize;
        let physical_address = physical_frame_address + frame_offset as usize;
        let result = page_table.translate(virtual_address);

        sanity_assert_translate_address_result(virtual_address, physical_address, result);
    }

    frame_alloc.free(physical_frame.address());
    page_table.unmap_page(virtual_frame);
}

unsafe fn paging_unmap_should_properly_unmap_elements(page_table : &mut page_table::P4Table, frame_alloc : &mut BuddyAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = Frame::from_address(frame_alloc.allocate(FRAME_SIZE).expect("No frames for paging test"));

    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT, frame_alloc);
    page_table.unmap_page(virtual_frame);

    let result = page_table.translate_page(virtual_frame);

    assert!(result.is_none(),
        "Translation of virtual page {} returned physical frame {} after unmap, but should return empty result",
        virtual_frame,
        result.unwrap());

    frame_alloc.free(physical_frame.address());
}

fn sanity_assert_translate_page_result(virtual_frame : Frame, physical_frame : Frame, result : Option<Frame>) {
    assert!(result.is_some(),
        "Returned empty result for translation of virtual frame {}",
        virtual_frame);

    let result_frame = result.unwrap();

    assert!(physical_frame == result_frame,
        "Returned invalid translation result for virtual frame {}. Should be frame {} but was {}",
        virtual_frame,
        physical_frame,
        result_frame);
}

fn sanity_assert_translate_address_result(virtual_address : usize, physical_address : usize, result : Option<usize>){
    assert!(result.is_some(),
        "Returned empty result for translation of virtual frame {}",
        virtual_address);

    let result_address = result.unwrap();

    assert_eq!(physical_address, result_address, "Returned invalid translation result for virtual frame {}. Should be frame {} but was {}", virtual_address, physical_address, result_address);
}

