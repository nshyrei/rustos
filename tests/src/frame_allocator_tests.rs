use memory::frame::frame_allocator::FrameAllocator;
use multiboot::multiboot_header::MultibootHeader;
use memory::frame::Frame;
use memory::kernel::bump_allocator::*;
use memory::kernel::KERNEL_BASIC_HEAP_ALLOCATOR;
use memory::kernel::frame_bitmap::*;
use std::mem;
use multiboot::multiboot_header::tags_info::{basic_memory_info, elf_sections, memory_map};

#[test]
fn should_properly_determine_first_memory_area() {
    let elf_section_entry_size = mem::size_of::<elf_sections::ElfSectionHeader>();
    let elf_sections_size = 5 * mem::size_of::<u32>() + elf_section_entry_size;
    let memory_map_entry_size = mem::size_of::<memory_map::MemoryMapEntry>();
    let memory_map_size = 4 * mem::size_of::<u32>() + memory_map_entry_size * 4;
    let multiboot_size = 2 * mem::size_of::<u32>() + memory_map_size + elf_sections_size;

    let bytes : [u32; 53] = [
        multiboot_size as u32,          // multiboot length
        1,  // multiboot reserved

        6,  // memory map type
        memory_map_size as u32,         // memory map size
        memory_map_entry_size as u32,  // memory map entry size
        1,   // memory map version

        12,  // [ memory map entry base addr
        0, // ]
        0,  // [ memory map entry length
        0, // ]
        1,  // memory map entry type
        1, // memory map entry reserved

        25,  
        0, 
        0,  
        0, 
        1,  
        1, 
        
        10,
        0,
        0, 
        0,
        1, 
        1,

        0,
        0,
        0, 
        12,
        0,  //entry type = 0 and thus should be ignored, despite having min base address
        1,
        
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
        
    let kernel_heap = [0;256];
    let addr = bytes.as_ptr() as usize;
    let kernel_heap_addr = kernel_heap.as_ptr() as usize;
    let multiboot_header1 = MultibootHeader::load(addr);
    unsafe { KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address(kernel_heap_addr); };

    let frame_allocator = FrameAllocator::new(multiboot_header1);

    assert!(frame_allocator.current_memory_area().base_address() == 10);    
}

#[test]
fn should_properly_determine_kernel_start_and_end_address() {
    let elf_section_entry_size = mem::size_of::<elf_sections::ElfSectionHeader>();
    let elf_sections_size = 5 * mem::size_of::<u32>() + 4 * elf_section_entry_size;
    let memory_map_entry_size = mem::size_of::<memory_map::MemoryMapEntry>();
    let memory_map_size = 4 * mem::size_of::<u32>() + memory_map_entry_size;
    let multiboot_size = 2 * mem::size_of::<u32>() + memory_map_size + elf_sections_size;

    let bytes : [u32; 83] = [
        multiboot_size as u32,          // multiboot length
        1,  // multiboot reserved

        6,  // memory map type
        memory_map_size as u32,         // memory map size
        memory_map_entry_size as u32,  // memory map entry size
        1,   // memory map version

        12,  // [ memory map entry base addr
        0, // ]
        0,  // [ memory map entry length
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
        10,  //[ address ]
        0,  //
        0,  // [ offset ]
        0,  //
        10,  // [ size ]
        0,  //
        0,  // link
        0,  // info
        0,  // [ address align ]
        0,  //
        0,  // [ entry size ]
        0,   //

        1,  //name
        0,  //section type = 0 should be ignored
        0,  //[ flags ]
        0,  //
        100,  //[ address ]
        0,  //
        0,  // [ offset ]
        0,  //
        100,  // [ size ]
        0,  //
        0,  // link
        0,  // info
        0,  // [ address align ]
        0,  //
        0,  // [ entry size ]
        0,   //

        1,  //name
        1,  //section type
        0,  //[ flags ]
        0,  //
        50,  //[ address ]
        0,  //
        0,  // [ offset ]
        0,  //
        50,  // [ size ]
        0,  //
        0,  // link
        0,  // info
        0,  // [ address align ]
        0,  //
        0,  // [ entry size ]
        0,   //

        1,  //name
        1,  //section type
        0,  //[ flags ]
        0,  //
        0,  //[ address ]
        0,  //
        0,  // [ offset ]
        0,  //
        10,  // [ size ]
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
        
    let kernel_heap = [0;256];
    let addr = bytes.as_ptr() as usize;
    let kernel_heap_addr = kernel_heap.as_ptr() as usize;
    let multiboot_header1 = MultibootHeader::load(addr);
    unsafe { KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address(kernel_heap_addr); };

    let frame_allocator = FrameAllocator::new(multiboot_header1);

    assert!(frame_allocator.kernel_start_frame() == Frame::from_address(0));
    assert!(frame_allocator.kernel_end_frame() == Frame::from_address(50 + 50));
}

#[test]
fn should_properly_determine_available_memory() {
    let elf_section_entry_size = mem::size_of::<elf_sections::ElfSectionHeader>();
    let elf_sections_size = 5 * mem::size_of::<u32>() + elf_section_entry_size;
    let memory_map_entry_size = mem::size_of::<memory_map::MemoryMapEntry>();
    let memory_map_size = 4 * mem::size_of::<u32>() + 4 * memory_map_entry_size;
    let multiboot_size = 2 * mem::size_of::<u32>() + memory_map_size + elf_sections_size;

    let bytes : [u32; 53] = [
        multiboot_size as u32,          // multiboot length
        1,  // multiboot reserved

        6,  // memory map type
        memory_map_size as u32,         // memory map size
        memory_map_entry_size as u32,  // memory map entry size
        1,   // memory map version

        12,  // [ memory map entry base addr
        0, // ]
        10,  // [ memory map entry length
        0, // ]
        1,  // memory map entry type
        1, // memory map entry reserved

        25,  
        0, 
        20,  
        0, 
        1,  
        1, 
        
        10,
        0,
        30, 
        0,
        1, 
        1,

        0,
        0,
        0, 
        12,
        0,  //entry type = 0 and thus should be ignored, despite having min base address
        1,
        
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
        
    let kernel_heap = [0;256];
    let addr = bytes.as_ptr() as usize;
    let kernel_heap_addr = kernel_heap.as_ptr() as usize;
    let multiboot_header1 = MultibootHeader::load(addr);
    unsafe { 
        KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address(kernel_heap_addr); 
    
        let before_test_ptr = KERNEL_BASIC_HEAP_ALLOCATOR.current_pointer();
        let test_bit_map = FrameBitMap::new(10 + 20 + 30, 4096);
        let after_test_ptr = KERNEL_BASIC_HEAP_ALLOCATOR.current_pointer();
        let test_bitmap_size = KERNEL_BASIC_HEAP_ALLOCATOR.current_pointer() - before_test_ptr;


        let frame_allocator = FrameAllocator::new(multiboot_header1);
        let frame_alloc_bitmap_size = KERNEL_BASIC_HEAP_ALLOCATOR.current_pointer() - after_test_ptr;

        // bump allocation size for bitmaps should be equal
        assert!(test_bitmap_size == frame_alloc_bitmap_size);
    };
}