#![feature(lang_items)]
#![feature(asm)]
#![feature(alloc)]
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

use core::fmt::Write;

#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "C" fn rust_main(multiboot_header_address: usize) {    
    unsafe {
        let multiboot_header = MultibootHeader::load(multiboot_header_address);
        let mut vga_writer = Writer::new();

        print_multiboot_data(multiboot_header, &mut vga_writer);
        remove_tail_should_properly_remove_tail_element();
        let mut frame_allocator = FrameAllocator::new(multiboot_header);
        //free_list_should_properly_set_free();
        //let mut buddy_allocator = BuddyAllocator::new(frame_allocator.end_address() + 1, 104857600);
        //frame_allocator.set_buddy_start(Frame::from_address(buddy_allocator.start_address()));
        //frame_allocator.set_buddy_end(Frame::from_address(buddy_allocator.end_address()));
        //buddy_allocator.allocate(1024);        
        let mut temp_p4_table = paging::p4_table();
        paging::remap_kernel(&mut temp_p4_table, &mut frame_allocator, multiboot_header);

        let p4_table = paging::p4_table();
        

        print_multiboot_data(multiboot_header, &mut vga_writer);

        //writeln!(&mut vga_writer, "{}", predefined_p4_table);

        // run pre-init tests
        paging_map_should_properly_map_pages(p4_table, &mut frame_allocator, &mut vga_writer);
        paging_translate_page_should_properly_translate_pages(p4_table, &mut frame_allocator);
        paging_unmap_should_properly_unmap_elements(p4_table, &mut frame_allocator);
        paging_translate_address_should_properly_translate_virtual_address(p4_table, &mut frame_allocator);        
    }
    loop {}
}

#[test]
pub fn take_first_free_block_should_properly_work() {    
    use stdx_memory::collections::double_linked_list::DoubleLinkedList;
    use stdx_memory::collections::double_linked_list::DoubleLinkedListCell;
    use stdx_memory::collections::double_linked_list::DoubleLinkedListIterator;
    use stdx::iterator::IteratorExt;
    use memory::allocator::free_list::FreeListAllocator;
    use memory::allocator::buddy::BuddyFreeList;
    
    let heap = [0;400];    
    
    let (heap_start, heap_size) =          (heap.as_ptr() as usize, 400);
    
    let size = mem::size_of::<DoubleLinkedListCell<u8>>();
    let mut allocator = FreeListAllocator::from_address(heap_start, 400, mem::size_of::<DoubleLinkedListCell<u8>>());
    let mut buddy_free_list = BuddyFreeList::new(2, 2, &mut allocator);

    // order is important, take first should return 2 not 0
    buddy_free_list.set_free(2, &mut allocator);
    buddy_free_list.set_free(0, &mut allocator);

    let result = buddy_free_list.first_free_block(&mut allocator);

    assert!(buddy_free_list.is_in_use(2), "Failed to set in use for first freed block. Should set in use for block starting at address {}",
        2);
    assert!(result.is_some(), "Failed to return first free block. List had blocks 2-0");
    assert!(result.unwrap() == 2, "Returned invalid first free block. Returned {}, but should be {}",
        result.unwrap(),
        2);
}

pub fn remove_tail_should_properly_remove_tail_element() {
    use core::mem;
    use stdx_memory::collections::double_linked_list::DoubleLinkedList;
    use stdx_memory::collections::double_linked_list::DoubleLinkedListCell;
    use stdx_memory::collections::double_linked_list::DoubleLinkedListIterator;
    use stdx::iterator::IteratorExt;
    let heap = [0;200];    
    
    let (heap_start, heap_size) =          (heap.as_ptr() as usize, 200);
        

    let size = mem::size_of::<DoubleLinkedListCell<u8>>();
    let mut allocator = BumpAllocator::from_address(heap_start, 200, mem::size_of::<DoubleLinkedListCell<u8>>());
    let mut list : DoubleLinkedList<u8> = DoubleLinkedList::new(&mut allocator);
        
    let values : [u8;3] = [1, 2, 3];
    let values_len = values.len();

    list.add_to_tail(values[0], &mut allocator);
    list.add_to_tail(values[1], &mut allocator); 
    list.add_to_tail(values[2], &mut allocator);

    list.remove_tail(&mut allocator);
    
    let count = DoubleLinkedListIterator::new(list.tail()).count();

    assert!(count == values_len - 1, "DoubleLinkedList::remove_tail didn't remove the last element. Iterator count returned {}, but should be {}",
        count,
        values_len - 1);

    let mut iter = DoubleLinkedListIterator::new(list.tail()).index_items();
    
    // todo : make a reverse indexing iterator
    while let Some((result, index)) = iter.next() {
        assert!(result == values[values_len - index - 2], "DoubleLinkedList::remove_tail didn't remove the last element. Value returned from iterator and reference differ. Was {}, but should be {}",
        result,
        values[values_len - index - 2]);        
    };      
}


#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt() -> ! {
    loop {}
}

fn print_multiboot_data(multiboot_header : &MultibootHeader, vga_writer : &mut Writer) {
    let mut memInfo1 = multiboot_header            
            .read_tag::<basic_memory_info::BasicMemoryInfo>()
            .unwrap();

    writeln!(vga_writer, "---Basic memory info---");
    writeln!(vga_writer, "{}", memInfo1);

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

    assert!(physical_address == result_address, 
        "Returned invalid translation result for virtual frame {}. Should be frame {} but was {}",
        virtual_address,
        physical_address,
        result_address);
}

