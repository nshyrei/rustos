pub mod page_table;

use paging::page_table::*;
use frame::frame_allocator::*;
use frame::Frame;
use hardware::x86_64::tlb;
use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags_info::elf_sections;

fn p4_table() -> &'static mut PageTable<P4> {
    const P4_TABLE_ADDRESS : usize = 0xfffffffffffff000; //recursive mapping to P4s 0 entry
    unsafe { &mut (*(P4_TABLE_ADDRESS as *mut PageTable<P4>)) }
}

pub fn modify_other<Action>(other_p4_table : &'static mut PageTable<P4>, frame_allocator : &mut FrameAllocator, action : Action)
where Action : FnOnce(&mut PageTable<P4>) {
    // save current p4 table address in current p4s 510 entry,
    // it could be any free entry but 510 is chosen just because its
    // below the recursive 511s entry

    let p4 = p4_table();

    // save physical address of current p4 in current p4's 510 entry

    let p4_physical_address = Frame::from_address(p4[511].address());   // p4's 511 entry points to self
    let temp_virtual_address = Frame::from_address(0xFF8000000000);     // should be p4's 510 entry    

    p4.map(temp_virtual_address, p4_physical_address, frame_allocator);

    // create temp p4
    //let mut new_p4 = new_page_table::<P4>(frame_allocator);
    let new_p4_physical_address = other_p4_table as *const _ as usize;    

    // set recursive entry for temp p4
    p4[511].set(new_p4_physical_address, EntryFlags::PRESENT | EntryFlags::WRITABLE);
    
    tlb::flush_all();

    action(other_p4_table);
    
    // place old recursive entry back into current p4
    let mut saved_p4 = unsafe { &mut (*(temp_virtual_address.address() as *mut PageTable<P4>)) };
    saved_p4[511].set_frame(p4_physical_address, EntryFlags::PRESENT | EntryFlags::WRITABLE);

    tlb::flush_all();

    // unmap recursive address saving
    p4.unmap(temp_virtual_address);
}

pub fn remap_kernel(multiboot_header : &'static MultibootHeader) {
    let elf_sections = multiboot_header
            .read_tag::<elf_sections::ElfSections>()
            .unwrap();

    let mut loaded_elf_sections = elf_sections.entries().filter(|e| e.flags().contains(elf_sections::ElfSectionFlags::ELF_SECTION_ALLOCATED));
        
    while let Some(e) = loaded_elf_sections.next() {
        
    }
}
