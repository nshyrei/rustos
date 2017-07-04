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
use memory::kernel::empty_frame_list::{EmptyFrameList, EmptyFrameListIterator};


static mut multiboot_header: Option<&'static MultibootHeader> = None;

#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "C" fn rust_main(multiboot_header_address: usize) {

    let mut vga_writer = Writer::new();

    unsafe {
        multiboot_header = Some(MultibootHeader::load(multiboot_header_address));
    }

    unsafe {
        let mut memInfo1 = multiboot_header
            .unwrap()
            .read_tag::<basic_memory_info::BasicMemoryInfo>();

        let memInfo = multiboot_header
            .unwrap()
            .read_tag::<memory_map::MemoryMap>();

        let elf_sections = multiboot_header
            .unwrap()
            .read_tag::<elf_sections::ElfSections>();
        let mut elf_sectionsIt = elf_sections.entries();

        while let Some(e) = elf_sectionsIt.next() {
            let ee = *e;
            let a = 0;
            writeln!(vga_writer,
                     "Address {} type {}",
                     e.address(),
                     e.section_type());
        }


        adding_elems_should_work_properly();
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

fn adding_elems_should_work_properly() {
    let bytes = [0; 256];
    let addr = bytes.as_ptr() as usize;
    let test_values = [0, 2, 3, 4, 12, 20, 44, 10];
    let test_values_len = test_values.len();

    let mut allocator = BumpAllocator::from_address(addr);
    let mut head = EmptyFrameList::new(test_values[0], &mut allocator);

    for i in 1..test_values_len {
        head = head.add(test_values[i], &mut allocator);
    }

    let it = EmptyFrameListIterator::new(head);
    let it_count = it.count();


    let mut iterator = EmptyFrameListIterator::new(head);
    let mut idx = test_values_len - 1;
    while let Some(e) = iterator.next() {

        idx -= 1;
    }
}
