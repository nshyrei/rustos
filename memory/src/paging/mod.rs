pub mod page_table;

use paging::page_table::{ P4Table, PAGE_TABLE_SIZE, PAGE_TABLE_ENTRY_SIZE };
use frame::frame_allocator::*;
use frame::Frame;
use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags::elf;
use hardware::x86_64::registers;
use stdx_memory::MemoryAllocator;

/// Returns size of page tables to describe virtual memory
/// # Arguments
/// * `virtual_memory_size` - the size of virtual memory
pub fn table_size_for(virtual_memory_size : usize) -> usize {
    (virtual_memory_size / PAGE_TABLE_SIZE) * PAGE_TABLE_ENTRY_SIZE
}

/// Returns current p4 table.
pub fn p4_table() -> &'static mut P4Table {
    const P4_TABLE_ADDRESS : usize = 0xfffffffffffff000;         // recursive mapping to P4s 0 entry
    unsafe { &mut (*(P4_TABLE_ADDRESS as *mut P4Table)) } // reading predefined recursive address is safe
}

/// Switches paging tables
/// # Arguments
/// * `new_p4_table_address` - physical address of new p4 table
/// # Why unsafe
///  Uses registers::cr3_write() which is unsafe
pub unsafe fn switch_tables(new_p4_table_address : usize) {
    registers::cr3_write(new_p4_table_address as u64);
}

/// Properly maps (with proper flags and placement) kernel frames like: 
/// kernel code, bump allocator, vga buffer etc. to fresh paging table. After that switches to
/// that table. 
/// # Arguments
/// * `current_p4_table` - current p4 table
/// * `frame_allocator` - frame allocator
/// * `multiboot_header` - multiboot header
/// # Why unsafe
///  Uses modify_other_table(), page_table.unmap() which are unsafe
pub unsafe fn remap_kernel(current_p4_table : &mut P4Table, frame_allocator : &mut FrameAllocator, multiboot_header : &MultibootHeader){     
    let new_p4_table_address = frame_allocator.allocate().expect("No frames for kernel remap");

    current_p4_table.modify_other_table(new_p4_table_address, 
        frame_allocator, 
        |p4, frame_alloc| remap_kernel0(p4, frame_alloc, multiboot_header));

    let old_p4_address = registers::cr3();

    switch_tables(new_p4_table_address.address());

    let new_p4 = p4_table();

    // old p4 table should be identity mapped as part of kernel elf section
    // unmapping it will create 'stack guard' - an unmapped area just below the stack.
    // Accessing it will immediately throw segfault, thus preventing stack growing out of hand
    // and overwriting something.
    new_p4.unmap(old_p4_address  as usize)
}

fn remap_kernel0<M>(p4_table : &mut P4Table, frame_allocator : &mut M, multiboot_header : & MultibootHeader)  where M : MemoryAllocator + FrameAllocatorFake {
    let elf_sections = multiboot_header
            .read_tag::<elf::ElfSections>()
            .unwrap();

    for multiboot_header_frame in Frame::range_inclusive(multiboot_header.start_address(), multiboot_header.end_address()){
        p4_table.map_page_1_to_1(multiboot_header_frame, page_table::PRESENT, frame_allocator);
    }    
        
    // todo figure out map or not non allocated section.
    // Reason: mapping non allocated section results in seg fault after remap operation.
    let mut loaded_elf_sections = elf_sections.entries().filter(|e| e.flags().contains(elf::ALLOCATED));
    while let Some(elf_section) = loaded_elf_sections.next() {
        for elf_frame in Frame::range_inclusive(elf_section.start_address() as usize, elf_section.end_address() as usize) {
            let page_flag = elf_sections_flag_to_page_flag(elf_section.flags());
            p4_table.map_page_1_to_1(elf_frame, page_flag, frame_allocator);
        }
    }

    let vga_frame = Frame::from_address(0xb8000);
    p4_table.map_page_1_to_1(vga_frame, page_table::PRESENT | page_table::WRITABLE, frame_allocator);

    // remap bump allocator    
    let bump_allocator = frame_allocator.bump_allocator();
    
    for bump_allocator_frame in Frame::range_inclusive(bump_allocator.start_address(), bump_allocator.end_address()) {
        p4_table.map_page_1_to_1(bump_allocator_frame, page_table::PRESENT | page_table::WRITABLE, frame_allocator);
    }
}

fn elf_sections_flag_to_page_flag(elf_flags : elf::ElfSectionFlags) -> page_table::EntryFlags {
    let mut result = page_table::EntryFlags::from_bits_truncate(0);

    if elf_flags.contains(elf::ALLOCATED) {
        result |= page_table::PRESENT;
    }

    if elf_flags.contains(elf::WRITABLE) {
        result |= page_table::WRITABLE;
    }

    if !elf_flags.contains(elf::EXECUTABLE) {
        // need to set NXE bit in some register for this to work,
        // it will throw page fault otherwise
        // result |= NO_EXECUTE;
    }

    result
}


