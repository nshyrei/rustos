use memory::paging::page_table::P4;
use memory::frame::Frame;
use memory::kernel::bump_allocator::BumpAllocator;
use memory::paging::simulation::*;
use multiboot::multiboot_header::MultibootHeader;
use memory::frame::frame_allocator::*;
use multiboot::multiboot_header::tags_info::{basic_memory_info, elf_sections, memory_map};
use std::mem;

#[test]
fn adding_elems_should_work_properly() {
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
    let mut p4_as_table = unsafe { &mut (*(p4_address as *mut TestPageTable<P4>)) };
    
    p4[511] = (p4_address as u64) << 12 | 1; // set recursive entry    
    
    let mut paging_simulation = PagingSimulation::new(p4_address);
    let mut bump_allocator = BumpAllocator::from_address(kernel_heap_addr);
    
    let mut frame_allocator = FrameAllocator::new(multiboot_header1, &mut bump_allocator);
    let virtual_frame = Frame::new(15);
    let physical_frame = Frame::new(340);

    map(p4_as_table, virtual_frame, physical_frame, &mut frame_allocator, &mut paging_simulation);

    paging_simulation.reset_address_savings();    

    let result = translate(p4_as_table, virtual_frame, &mut paging_simulation);
    let a= 0;
}