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
use memory::paging::page_table::*;


static mut multiboot_header: Option<&'static MultibootHeader> = None;

#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "C" fn rust_main(multiboot_header_address: usize) {    

    unsafe {
        multiboot_header = Some(MultibootHeader::load(multiboot_header_address));        
    }

    unsafe {
        let mut vga_writer = Writer::new();
        let mut memInfo1 = multiboot_header
            .unwrap()
            .read_tag::<basic_memory_info::BasicMemoryInfo>()
            .unwrap();

        writeln!(vga_writer, "---Basic memory info---");
        writeln!(vga_writer, "{}", memInfo1);

        let memInfo = multiboot_header
            .unwrap()
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
            .unwrap()
            .read_tag::<elf_sections::ElfSections>()
            .unwrap();
        let mut elf_sectionsIt = elf_sections.entries();

        writeln!(vga_writer, "---Elf sections---");
        while let Some(e) = elf_sectionsIt.next() {
            writeln!(vga_writer, "{}", e);
        }


        bitmap_new_should_create_empty_bitmap_of_size_zero_if_frame_count_is_inside_bitmap_entry_size();
        let a = 0;
    }



    let hello = b"Hello World!";
    let color_byte = 0x1f; // white foreground, blue background

    let mut hello_colored = [color_byte; 24];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i * 2] = *char_byte;
    }

    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt() -> ! {
    loop {}
}


fn bitmap_new_should_create_empty_bitmap_of_size_zero_if_frame_count_is_inside_bitmap_entry_size
    () {

    let free_memory : [u64; 3072] = [0; 3072];
    let free_memory_start_address = free_memory.as_ptr() as u32;

    let elf_section_entry_size = mem::size_of::<elf_sections::ElfSectionHeader>();
    let elf_sections_size = 5 * mem::size_of::<u32>() + elf_section_entry_size;
    let memory_map_entry_size = mem::size_of::<memory_map::MemoryMapEntry>();
    let memory_map_size = 4 * mem::size_of::<u32>() + memory_map_entry_size;
    let multiboot_size = 2 * mem::size_of::<u32>() + memory_map_size + elf_sections_size;

    let bytes : [u32; 35] = [
        multiboot_size as u32,          // multiboot length
        1,  // multiboot reserved

        6,  // memory map type
        memory_map_size as u32,         // memory map size
        memory_map_entry_size as u32,  // memory map entry size
        1,   // memory map version

        free_memory_start_address,  // [ memory map entry base addr
        0, // ]
        24576,  // [ memory map entry length
        0, // ]
        1,  // memory map entry type
        1, // memory map entry reserved        
        
        9,  // elf
        elf_sections_size as u32,        
        1,   // entries num
        elf_section_entry_size as u32,
        1,  //shndx

        1,  //name
        1,  //section type
        0,  //[ flags ]
        0,  //
        0,  //[ address ]
        0,  //
        0,  // [ offset ]
        0,  //
        0,  // [ size ]
        0,  //
        0,  // link
        0,  // info
        0,  // [ address align ]
        0,  //
        0,  // [ entry size ]
        0,   //

        0,  // end tag
        0,  // 
        
        ];
    

    let addr = bytes.as_ptr() as usize;
    let multiboot_header1 = MultibootHeader::load(addr);

    let mut stub_p4 : [u64; 512] = [0; 512];
    let mut p4 : [u64; 512] = [0; 512];
    let mut p3 : [u64; 512] = [0; 512];
    let mut p2 : [u64; 512] = [0; 512];
    let mut p1 : [u64; 512] = [0; 512];
    let mut address_savings : [u64; 3] = [0; 3];
    

    let kernel_heap = [0;256];    
    let kernel_heap_addr = kernel_heap.as_ptr() as usize;
    let stub_p4_address = stub_p4.as_ptr() as usize;

    let p4_address = p4.as_ptr() as usize;
    p4[511] = (p4_address as u64) << 12 | 1; // set recursive entry
    address_savings[0] = 0x0000fffffffff000;
    
    let mut bump_allocator = BumpAllocator::from_address(kernel_heap_addr);
    let p4_as_table = unsafe { &mut (*(p4_address as *mut TestPageTable<P4>)) };
    let mut frame_allocator = FrameAllocator::new(multiboot_header1, &mut bump_allocator);
    let virtual_frame = Frame::new(1);
    let physical_frame = Frame::new(2);

    map_test(stub_p4_address, virtual_frame, physical_frame, &mut frame_allocator, p4_as_table, &mut address_savings);
    let a= 0;
}
