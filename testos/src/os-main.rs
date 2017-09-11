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
    use core::ptr;

    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = frame_alloc.allocate().expect("No frames for paging test");
        
    page_table::map(virtual_frame, physical_frame, frame_alloc);

    // check that translation result and raw pointer deref point to same data (prob rethink the test)
    let virtual_frame_as_ptr = virtual_frame.address() as *mut u32;
    unsafe { ptr::write(virtual_frame_as_ptr, u32::max_value()); }

    let result = page_table::translate(virtual_frame);

    assert_paging_map(virtual_frame, physical_frame, result);

    frame_alloc.deallocate(physical_frame);
    page_table::unmap(virtual_frame);
}

fn assert_paging_map(virtual_frame : Frame, physical_frame : Frame, result : Option<Frame>) {
    
    assert!(result.is_some(), 
        "Returned empty result for translation of virtual frame {}", 
        virtual_frame);
        
    assert!(physical_frame == result.unwrap(), 
        "Returned invalid translation result for virtual frame {}. Should be frame {} but was {}",
        virtual_frame,
        physical_frame,
        result.unwrap());

    unsafe {
        let result_data = *(result.unwrap().address() as *const u32);
        let raw_ptr_data = *(virtual_frame.address() as *const u32);

        assert!(result_data == raw_ptr_data, 
            "Translate and raw pointer read produce different results. Virtual frame {}. Translation points to u32 of value {}, but raw pointer points to {}",
            virtual_frame,
            result_data,
            raw_ptr_data);
    }            
}

fn paging_unmap_should_properly_unmap_elements(frame_alloc : &mut FrameAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = frame_alloc.allocate().expect("No frames for paging test");

    page_table::map(virtual_frame, physical_frame, frame_alloc);    
    page_table::unmap(virtual_frame);    

    let result = page_table::translate(virtual_frame);

    assert!(result.is_none(),
        "Translation of virtual page {} returned physical frame {} after unmap, but should return empty result",
        virtual_frame,
        result.unwrap());

    frame_alloc.deallocate(physical_frame);
}