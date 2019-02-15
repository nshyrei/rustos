#![feature(lang_items)]
#![feature(asm)]
#![feature(alloc)]
#![feature(discriminant_value)]
#![no_std]


extern crate rlibc;
extern crate multiboot;
extern crate display;
extern crate memory;
extern crate hardware;
extern crate alloc;
extern crate malloc;
extern crate stdx_memory;
extern crate stdx;

use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags::{basic_memory_info, elf, memory_map};
use multiboot::multiboot_header::tags::memory_map::*;
use display::vga::writer::Writer;
use memory::allocator::bump::BumpAllocator;
use memory::frame::frame_allocator::*;
use memory::frame::Frame;
use memory::frame::FRAME_SIZE;
use memory::paging;
use memory::paging::page_table;
use memory::paging::page_table::P4Table;
use stdx_memory::MemoryAllocator;
use hardware::x86_64::registers;
use core::clone::Clone;
use core::fmt::Write;
use alloc::boxed::Box;
use malloc::TestAllocator;
use stdx_memory::collections::immutable::double_linked_list::DoubleLinkedList;
use memory::allocator::slab::SlabAllocator;
use memory::allocator::slab::SlabHelp;
use stdx_memory::heap::RC;
use stdx_memory::heap::SharedBox;
use core::ptr::NonNull;
use alloc::alloc::Layout;
use stdx_memory::heap;

static mut vga_writerg : Option<Writer> = None;

#[global_allocator]
static mut HEAP_ALLOCATOR: SlabHelp = SlabHelp { value : None };

#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "C" fn rust_main(multiboot_header_address: usize) {    
    unsafe {        

        
        let multiboot_header = MultibootHeader::load(multiboot_header_address);

        let mut vga_writer = Writer::new();

        print_multiboot_data(multiboot_header, &mut vga_writer);
        
        let mut frame_allocator = FrameAllocator::new(multiboot_header);

        paging::remap_kernel(&mut paging::p4_table(), &mut frame_allocator, multiboot_header);

        print_multiboot_data(multiboot_header, &mut vga_writer);

        let mut slab_allocator = slab_allocator_should_be_fully_free(&mut vga_writer, 0);
        HEAP_ALLOCATOR.value = NonNull::new(&mut slab_allocator as *mut SlabAllocator);

        {

            let us : usize = 128;
            let sz = core::mem::size_of::<usize>();
            let boxin = Box::new(us);
            let bb = boxin;

        }

        //writeln!(&mut vga_writer, "{}", predefined_p4_table);

        // run pre-init tests
        let p4_table = paging::p4_table();

        paging_map_should_properly_map_pages(p4_table, &mut frame_allocator, &mut vga_writer);
        paging_translate_page_should_properly_translate_pages(p4_table, &mut frame_allocator);
        paging_unmap_should_properly_unmap_elements(p4_table, &mut frame_allocator);
        paging_translate_address_should_properly_translate_virtual_address(p4_table, &mut frame_allocator);        
    }
    loop {}
}

pub fn slab_allocator_should_be_fully_free(writer: &mut Writer, adr: usize) -> SlabAllocator {
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

    let frame_allocator_start = Frame::address_align_up(heap_addr);

    let allocator = BuddyAllocator::new(frame_allocator_start, frame_allocator_start + size);
    let slab_allocator_start = allocator.end_address() + 1;

    let mut slab_allocator = SlabAllocator::new(slab_allocator_start, frame_allocator_start + size, allocator);

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
    loop {}
}
#[lang = "oom"]
#[no_mangle]
pub extern "C" fn oom(l: Layout) -> ! {
    loop {}
}

fn print_multiboot_data(multiboot_header : &MultibootHeader, vga_writer : &mut Writer) {
    writeln!(vga_writer, "---Basic memory info---");

    let memInfo = multiboot_header            
            .read_tag::<memory_map::MemoryMap>()
            .unwrap();

    let mut mem_sections =
            memInfo
                .entries()
                .filter(|e| e.entry_type() == memory_map::MemoryMapEntryType::Available as u32);

    writeln!(vga_writer, "---Available memory {}", memInfo.available_memory());
    writeln!(vga_writer, "---Memory sections---");
    while let Some(e) = mem_sections.next() {
        writeln!(vga_writer, "{}", e);
    }
    
    let elf_sections = multiboot_header
            .read_tag::<elf::ElfSections>()
            .unwrap();
    let mut elf_sectionsIt = elf_sections.entries();

    writeln!(vga_writer, "---Elf sections---");
    writeln!(vga_writer, "Elf sections start: {}", elf_sections.entries_start_address().unwrap());
    writeln!(vga_writer, "Elf sections end: {}", elf_sections.entries_end_address().unwrap());

    while let Some(e) = elf_sectionsIt.next() {
        writeln!(vga_writer, "{}", e);
    }
}

unsafe fn paging_map_should_properly_map_pages(page_table : &mut page_table::P4Table, frame_alloc : &mut FrameAllocator, vga_writer : &mut Writer) {    

    let virtual_frame = Frame::from_address(0x400000000000);
    let physical_frame = frame_alloc.allocate().expect("No frames for paging test");
        
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

    frame_alloc.deallocate(physical_frame);
    page_table.unmap_page(virtual_frame);
}

unsafe fn paging_translate_page_should_properly_translate_pages(page_table : &mut page_table::P4Table, frame_alloc : &mut FrameAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = frame_alloc.allocate().expect("No frames for paging test");
        
    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT, frame_alloc);

    let result = page_table.translate_page(virtual_frame);

    sanity_assert_translate_page_result(virtual_frame, physical_frame, result);

    frame_alloc.deallocate(physical_frame);
    page_table.unmap_page(virtual_frame);
}

unsafe fn paging_translate_address_should_properly_translate_virtual_address(page_table : &mut page_table::P4Table, frame_alloc : &mut FrameAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = frame_alloc.allocate().expect("No frames for paging test");
        
    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT, frame_alloc);
    
    let virtual_frame_address = virtual_frame.address();
    let physical_frame_address = physical_frame.address();

    for frame_offset in 0..FRAME_SIZE {
        let virtual_address  = virtual_frame_address + frame_offset as usize;
        let physical_address = physical_frame_address + frame_offset as usize;
        let result = page_table.translate(virtual_address);        
                        
        sanity_assert_translate_address_result(virtual_address, physical_address, result);        
    }
    
    frame_alloc.deallocate(physical_frame);
    page_table.unmap_page(virtual_frame);
}

unsafe fn paging_unmap_should_properly_unmap_elements(page_table : &mut page_table::P4Table, frame_alloc : &mut FrameAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = frame_alloc.allocate().expect("No frames for paging test");

    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT, frame_alloc);    
    page_table.unmap_page(virtual_frame);    

    let result = page_table.translate_page(virtual_frame);

    assert!(result.is_none(),
        "Translation of virtual page {} returned physical frame {} after unmap, but should return empty result",
        virtual_frame,
        result.unwrap());

    frame_alloc.deallocate(physical_frame);
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

