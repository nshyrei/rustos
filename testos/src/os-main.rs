#![feature(lang_items)]
#![feature(asm)]
#![no_std]


extern crate rlibc;
extern crate multiboot;

#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "C" fn rust_main(multiboot_header: usize) {

    unsafe {
        let mut memInfo1 = multiboot::multiboot_header::memory_map(multiboot_header);

        let memInfo = multiboot::multiboot_header::basic_memory_info(multiboot_header);
        let mut elf_sections = multiboot::multiboot_header::elf_sections(multiboot_header);
        let elf_sections1 = multiboot::multiboot_header::elf_sections1(multiboot_header);
        let mut sectionIt = elf_sections1.sections();
        let mut elfSectionsIt = elf_sections
            .entries()
            .filter(|section| section.section_type != 0);
    }


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
