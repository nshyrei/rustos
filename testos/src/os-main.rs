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
use memory::kernel::frame_bitmap::FrameBitMap;


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

        writeln!(vga_writer, "---Basic memory info---");
        writeln!(vga_writer, "{}", memInfo1);

        let memInfo = multiboot_header
            .unwrap()
            .read_tag::<memory_map::MemoryMap>();

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
            .read_tag::<elf_sections::ElfSections>();
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
    let default_memory_value = 10;
    let bytes: [u8; 16] = [default_memory_value; 16];
    let addr = bytes.as_ptr() as usize;

    let ref mut allocator = BumpAllocator::from_address(addr);
    // frame size = 1 byte
    // available memory = 16 byte
    // bitmap entry holds 8 frame entries
    // 2 bitmap entries should be created
    let bitmap = FrameBitMap::new(16, 1, allocator);

    for i in 0..16 {
        bitmap.set_in_use(i);
    }

    let a = 0;
    // all two entries should contain only 1s, thus resulting in u8::max_value()

}
