pub mod page_table;

use paging::page_table::{P4, PageTable};
use frame::frame_allocator::*;
use frame::Frame;
use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags::elf;
use hardware::x86_64::tlb;
use hardware::x86_64::registers;

pub type P4Table = PageTable<P4>;

/// Returns current p4 table.
pub fn p4_table() -> &'static mut P4Table {
    const P4_TABLE_ADDRESS : usize = 0xfffffffffffff000;  // recursive mapping to P4s 0 entry
    unsafe { &mut (*(P4_TABLE_ADDRESS as *mut P4Table)) } // reading predefined recursive address is safe
}

/// Performs action on another p4 table through current p4 table.
/// # Arguments
/// * `current_p4_table` - current p4 table
/// * `other_p4_table_address` - frame that holds another p4 table
/// * `frame_allocator` - frame allocator
/// * `action` - function to be executed on another p4 table
/// # Why unsafe
///  Uses tlb::flush() which is unsafe
pub unsafe fn modify_other_table<F>(other_p4_table_address : Frame, frame_allocator : &mut FrameAllocator, action : F)
where F : FnOnce(&mut P4Table, &mut FrameAllocator)
{
    let current_p4_table = p4_table();
    // 1# map some unused virtual address to point to current p4
    // 2# map some unused virtual address to point to temp p4
    // 3# set recursive entry in temp p4
    // 4# unmap temp p4
    // 5# set recursive entry in current p4 to point to temp p4, this will
    //    make magical address '0xfffffffffffff000' point to temp table (thus not breaking any logic associated with that address)
    // 6# perform modifications on temp4
    // 7# read current p4 through temp virtual address defined in #1
    // 8# restore recursive entry in current p4
    // 9# unmap temp virtual address    

    // map some unused virtual address to point to current p4
    // this will be used to restore recursive mapping in current p4
    // after all the operations with temp p4 
    let p4_physical_address = Frame::from_address(current_p4_table[511].address());   // p4's 511 entry points to self
    let current_p4_save_address = Frame::from_address(0x400000000000);    // some temp address to save current p4
    current_p4_table.map_page(current_p4_save_address, p4_physical_address, page_table::PRESENT | page_table::WRITABLE, frame_allocator);
    
    // map temp table
    let temp_p4_virtual_address = Frame::from_address(0x200000000000);   // some temp address to map temp p4
    current_p4_table.map_page(temp_p4_virtual_address, other_p4_table_address, page_table::PRESENT, frame_allocator);
    
    // set recursive entry in temp table
    let temp_p4 = &mut (*(0x200000000000 as *mut P4Table));
    temp_p4.clear_all_entries();
    temp_p4.set_recursive_entry(other_p4_table_address, page_table::PRESENT | page_table::WRITABLE);
    
    current_p4_table.unmap_page(temp_p4_virtual_address);

    // set recursive entry of the current p4 to point to temp table
    current_p4_table.set_recursive_entry(other_p4_table_address, page_table::PRESENT | page_table::WRITABLE);
    
    tlb::flush_all();

    action(current_p4_table, frame_allocator); // reading recursive entry again will move us to the temp table
    
    // read old p4 and place recursive entry back
    let saved_p4 = &mut (*(current_p4_save_address.address() as *mut P4Table));
    saved_p4.set_recursive_entry(p4_physical_address, page_table::PRESENT | page_table::WRITABLE);
    
    // unmap recursive address saving
    current_p4_table.unmap_page(current_p4_save_address);

    tlb::flush_all();
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
pub unsafe fn remap_kernel(frame_allocator : &mut FrameAllocator, multiboot_header : &MultibootHeader){     
    let new_p4_table_address = frame_allocator.allocate().expect("No frames for kernel remap");

    modify_other_table(new_p4_table_address, 
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

fn remap_kernel0(p4_table : &mut P4Table, frame_allocator : &mut FrameAllocator, multiboot_header : & MultibootHeader) {
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


