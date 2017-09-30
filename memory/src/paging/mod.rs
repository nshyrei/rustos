pub mod page_table;

use paging::page_table::*;
use frame::frame_allocator::*;
use frame::Frame;
use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags_info::elf_sections;
use hardware::x86_64::tlb;
use hardware::x86_64::registers;
use kernel::bump_allocator;

pub type P4Table = PageTable<P4>;

pub fn modify_other<Action>(other_p4_table_address : usize, action : Action)
where Action : FnOnce(&mut PageTable<P4>) {
    // save current p4 table address in current p4s 510 entry,
    // it could be any free entry but 510 is chosen just because its
    // below the recursive 511s entry

    let p4 = p4_table();

    // save physical address of current p4 in current p4's 510 entry

    let p4_physical_address = Frame::from_address(p4[511].address());   // p4's 511 entry points to self    

    p4[510].set_frame(p4_physical_address, PRESENT | WRITABLE);
    //p4.map(temp_virtual_address, p4_physical_address, PRESENT | WRITABLE, frame_allocator);
    
    // set recursive entry for temp p4
    p4[511].set(other_p4_table_address, PRESENT | WRITABLE);
    
    tlb::flush_all();

    action(p4_table()); // reading recursive entry again will move us to the temp table
    
    // place old recursive entry back into current p4
    let mut saved_p4 = unsafe { &mut (*(0xFF8000000000 as *mut PageTable<P4>)) }; // 0xFF8000000000 should be p4's 510 entry
    saved_p4[511].set_frame(p4_physical_address, PRESENT | WRITABLE);

    tlb::flush_all();

    // unmap recursive address saving
    //p4.unmap(temp_virtual_address);
    p4[510].set_unused()
}

pub fn switch_tables(other_p4_table_address : usize) {
    unsafe { registers::cr3_write(other_p4_table_address as u64); }
}

pub fn remap_kernel(new_p4_table_address : usize, frame_allocator : &mut FrameAllocator, multiboot_header : &'static MultibootHeader){
    modify_other(new_p4_table_address, |table| remap_kernel0(table, frame_allocator, multiboot_header));
    switch_tables(new_p4_table_address);
}

pub fn p4_table() -> &'static mut PageTable<P4> {
    const P4_TABLE_ADDRESS : usize = 0xfffffffffffff000; //recursive mapping to P4s 0 entry
    unsafe { &mut (*(P4_TABLE_ADDRESS as *mut PageTable<P4>)) }
}

fn remap_kernel0(p4_table : &mut PageTable<P4>, frame_allocator : &mut FrameAllocator, multiboot_header : &'static MultibootHeader) {
    let elf_sections = multiboot_header
            .read_tag::<elf_sections::ElfSections>()
            .unwrap();

    let mut loaded_elf_sections = elf_sections.entries().filter(|e| e.flags().contains(elf_sections::ELF_SECTION_ALLOCATED));
        
    while let Some(elf_section) = loaded_elf_sections.next() {
        for elf_frame in Frame::range_inclusive(elf_section.start_address() as usize, elf_section.end_address() as usize) {
            p4_table.map_1_to_1(elf_frame, PRESENT, frame_allocator); // todo determine correct flag based on elf section flag
        }
    }

    let vga_frame = Frame::from_address(0xb8000);
    p4_table.map_1_to_1(vga_frame, PRESENT | WRITABLE, frame_allocator);

    for bump_heap_frame in Frame::range_inclusive(bump_allocator::HEAP_START, bump_allocator::HEAP_END){
        p4_table.map_1_to_1(bump_heap_frame, PRESENT | WRITABLE, frame_allocator);
    }
}
