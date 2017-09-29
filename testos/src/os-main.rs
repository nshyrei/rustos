#![feature(lang_items)]
#![feature(asm)]
#![no_std]


extern crate rlibc;
extern crate multiboot;
extern crate display;
extern crate memory;

use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags_info::{basic_memory_info, elf_sections, memory_map};
use display::vga::writer::Writer;
use core::fmt::Write;
use memory::kernel::bump_allocator::BumpAllocator;
use memory::frame::frame_allocator::*;
use memory::frame::Frame;
use memory::frame::FRAME_SIZE;
use memory::paging::page_table;

#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "C" fn rust_main(multiboot_header_address: usize) {    
    
        let multiboot_header = MultibootHeader::load(multiboot_header_address);
        
        let mut vga_writer = Writer::new();
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

        writeln!(vga_writer, "---Memory sections---");
        while let Some(e) = mem_sections.next() {
            writeln!(vga_writer, "{}", e);
        }
    
        let elf_sections = multiboot_header
            .read_tag::<elf_sections::ElfSections>()
            .unwrap();
        let mut elf_sectionsIt = elf_sections.entries();

        writeln!(vga_writer, "---Elf sections---");
        while let Some(e) = elf_sectionsIt.next() {
            writeln!(vga_writer, "{}", e);
        }
        
        let mut bump_allocator = BumpAllocator::new();
        let mut frame_allocator = FrameAllocator::new(multiboot_header, &mut bump_allocator);

        // run pre-init tests
        
        paging_map_should_properly_map_pages(&mut frame_allocator);
        paging_translate_page_should_properly_translate_pages(&mut frame_allocator);
        paging_unmap_should_properly_unmap_elements(&mut frame_allocator);
        paging_translate_address_should_properly_translate_virtual_address(&mut frame_allocator);
        
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt() -> ! {
    loop {}
}

fn paging_map_should_properly_map_pages(frame_alloc : &mut FrameAllocator) {    

    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = frame_alloc.allocate().expect("No frames for paging test");
        
    page_table::map(virtual_frame, physical_frame, frame_alloc);

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
    page_table::unmap(virtual_frame);
}

fn paging_translate_page_should_properly_translate_pages(frame_alloc : &mut FrameAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = frame_alloc.allocate().expect("No frames for paging test");
        
    page_table::map(virtual_frame, physical_frame, frame_alloc);

    let result = page_table::translate_page(virtual_frame);

    sanity_assert_translate_page_result(virtual_frame, physical_frame, result);

    let virtual_frame_address  = virtual_frame.address();
    let physical_frame_address = result.unwrap().address();
    for frame_offset in 0..FRAME_SIZE {        
        let virtual_address = virtual_frame_address + frame_offset as usize;
        let physical_address = physical_frame_address + frame_offset as usize;
        //ptr::write(virtual_address as *mut u8, u8::max_value());                                                
        assert_that_virtual_and_physical_address_point_to_same_data(virtual_address, physical_address);        
    }

    frame_alloc.deallocate(physical_frame);
    page_table::unmap(virtual_frame);
}

fn paging_translate_address_should_properly_translate_virtual_address(frame_alloc : &mut FrameAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = frame_alloc.allocate().expect("No frames for paging test");
        
    page_table::map(virtual_frame, physical_frame, frame_alloc);
    
    let virtual_frame_address = virtual_frame.address();
    let physical_frame_address = physical_frame.address();

    for frame_offset in 0..FRAME_SIZE {
        let virtual_address  = virtual_frame_address + frame_offset as usize;
        let physical_address = physical_frame_address + frame_offset as usize;
        let result = page_table::translate(virtual_address);
        //unsafe { ptr::write(virtual_address as *mut u8, u8::max_value()); }
                        
        sanity_assert_translate_address_result(virtual_address, physical_address, result);
        assert_that_virtual_and_physical_address_point_to_same_data(virtual_address, result.unwrap());    
    }
    
    frame_alloc.deallocate(physical_frame);
    page_table::unmap(virtual_frame);
}

fn paging_unmap_should_properly_unmap_elements(frame_alloc : &mut FrameAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = frame_alloc.allocate().expect("No frames for paging test");

    page_table::map(virtual_frame, physical_frame, frame_alloc);    
    page_table::unmap(virtual_frame);    

    let result = page_table::translate_page(virtual_frame);

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

fn assert_that_virtual_and_physical_address_point_to_same_data(virtual_address : usize, physical_address : usize) {    
    unsafe {
        let result_data  = *((physical_address) as *const u8);
        let raw_ptr_data = *((virtual_address) as *const u8);
                
        assert!(result_data == raw_ptr_data, 
            "Translated and raw pointer reads produce different results. Virtual address {}. Translation points to u8 of value {}, but raw pointer points to {}",
            virtual_address,
            result_data,
            raw_ptr_data);        
    }    
}