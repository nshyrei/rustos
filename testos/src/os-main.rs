#![feature(lang_items)]
#![feature(asm)]
#![no_std]


extern crate rlibc;
extern crate multiboot;
extern crate display;

use multiboot::multiboot_header;
use multiboot::multiboot_header::tags_info::{basic_memory_info, elf_sections, memory_map};
use display::vga::writer::Writer;

#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "C" fn rust_main(multiboot_header: usize) {

    unsafe {
        let mut memInfo1 =
            multiboot_header::read_tag::<basic_memory_info::BasicMemoryInfo>(multiboot_header);

        let memInfo = multiboot_header::read_tag::<memory_map::MemoryMap>(multiboot_header);

        let elf_sections =
            multiboot::multiboot_header::read_tag::<elf_sections::ElfSections>(multiboot_header);
        let mut elf_sectionsIt = elf_sections.entries();

        while let Some(e) = elf_sectionsIt.next() {
            let a = e.address;
            let a = 0;
        }
        //let proper_elf_sections = multiboot::multiboot_header::elf_sections1(multiboot_header);
        //let mut proper_elf_It = proper_elf_sections.sections();

        //while let Some(e) = proper_elf_It.next() {
        //  let a = 0;
        //}


        //let mut elf_sectionsIt = elf_sections.entries();

        //while let Some(e) = elf_sectionsIt.next() {
        //  let a = 0;
        //}

        let a = 0;
    }

    let mut vga_writer = Writer::new();
    let hello_string = "Hello World!";
    vga_writer.print_string(hello_string);
    vga_writer.println_string("hello_string");


    let hello = b"Hello World!";
    let color_byte = 0x1f; // white foreground, blue background

    let mut hello_colored = [color_byte; 24];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i * 2] = *char_byte;
    }

    // write `Hello World!` to the center of the VGA text buffer
    let buffer_ptr = (0xb8000 + 1988) as *mut _;
    unsafe { *buffer_ptr = hello_colored };

    loop {}

}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt() -> ! {
    loop {}
}
