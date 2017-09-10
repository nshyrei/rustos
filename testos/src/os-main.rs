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
use core::mem;
use memory::kernel::bump_allocator::BumpAllocator;
use memory::kernel::empty_frame_list::{EmptyFrameList, EmptyFrameListIterator};
use memory::kernel::frame_bitmap::FrameBitMap;
use memory::frame::frame_allocator::*;
use memory::frame::Frame;
use memory::paging::page_table;
use memory::frame::FRAME_SIZE;

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

        paging_map_should_properly_add_elements(&mut frame_allocator);
        paging_unmap_should_properly_unmap_elements(&mut frame_allocator); 
    

    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt() -> ! {
    loop {}
}


fn paging_map_should_properly_add_elements(frame_alloc : &mut FrameAllocator) {
    let virtual_frame = Frame::new(15);
    let physical_frame = Frame::new(340);
    
    page_table::map(virtual_frame, physical_frame, frame_alloc);    
        

    let result = page_table::translate(virtual_frame);

    assert!(result.is_some(), 
        "Returned empty result for translation of virtual frame {}", 
        virtual_frame);
        
    assert!(physical_frame == result.unwrap(), 
        "Returned invalid translation result for virtual frame {}. Should be frame {} but was {}",
        virtual_frame,
        physical_frame,
        result.unwrap());

    page_table::unmap(virtual_frame);
}

fn paging_unmap_should_properly_unmap_elements(frame_alloc : &mut FrameAllocator) {
    let virtual_frame = Frame::new(15);
    let physical_frame = Frame::new(340);
    
    page_table::map(virtual_frame, physical_frame, frame_alloc);    

    page_table::unmap(virtual_frame);

    let result = page_table::translate(virtual_frame);

    assert!(result.is_none(), 
        "Returned {} result for translation of virtual frame {} after it was unmapped", 
        result.unwrap(),
        virtual_frame);        
}