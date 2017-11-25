use memory::frame::frame_allocator::FrameAllocator;
use multiboot::multiboot_header::MultibootHeader;
use memory::frame::Frame;
use memory::frame::FRAME_SIZE;
use memory::kernel::bump_allocator::*;
use memory::kernel::frame_bitmap::*;
use std::mem;
use multiboot::multiboot_header::tags::{basic_memory_info, elf, memory_map};

#[test]
fn should_properly_determine_first_memory_area() {
    let elf_section_entry_size = mem::size_of::<elf::ElfSectionHeader>();
    let elf_size = 5 * mem::size_of::<u32>() + elf_section_entry_size;
    let memory_map_entry_size = mem::size_of::<memory_map::MemoryMapEntry>();
    let memory_map_size = 4 * mem::size_of::<u32>() + memory_map_entry_size * 3;
    let multiboot_size = 2 * mem::size_of::<u32>() + memory_map_size + elf_size;

    let bytes : [u32; 47] = [
        multiboot_size as u32,          // multiboot length
        1,  // multiboot reserved

        6,  // memory map type
        memory_map_size as u32,         // memory map size
        memory_map_entry_size as u32,  // memory map entry size
        1,   // memory map version

        FRAME_SIZE as u32,  // [ memory map entry base addr
        0, // ]
        (FRAME_SIZE * 2) as u32,  // [ memory map entry length
        0, // ]
        1,  // memory map entry type
        1, // memory map entry reserved

        (FRAME_SIZE * 3) as u32,  
        0, 
        10000,  
        0, 
        1,  
        1,        

        0,
        0,
        FRAME_SIZE as u32, 
        12,
        0,  //entry type = 0 and thus should be ignored, despite having min base address
        1,
        
        9,  // elf
        elf_size as u32,        
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
        1,  // [ size ]
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
    let KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address(kernel_heap_addr, 256);

    let frame_allocator = FrameAllocator::new_test(multiboot_header1, KERNEL_BASIC_HEAP_ALLOCATOR);

    assert!(frame_allocator.current_memory_area().base_address() == FRAME_SIZE as u64, 
        "Frame allocator failed to determine proper first memory area, determined base address was {}, but should be {}. Frame allocator {}",
        frame_allocator.current_memory_area().base_address(),
        FRAME_SIZE,
        frame_allocator);    
}

#[test]
fn should_properly_determine_kernel_start_and_end_address() {
    let elf_section_entry_size = mem::size_of::<elf::ElfSectionHeader>();
    let elf_size = 5 * mem::size_of::<u32>() + 4 * elf_section_entry_size;
    let memory_map_entry_size = mem::size_of::<memory_map::MemoryMapEntry>();
    let memory_map_size = 4 * mem::size_of::<u32>() + memory_map_entry_size;
    let multiboot_size = 2 * mem::size_of::<u32>() + memory_map_size + elf_size;

    let bytes : [u32; 83] = [
        multiboot_size as u32,          // multiboot length
        1,  // multiboot reserved

        6,  // memory map type
        memory_map_size as u32,         // memory map size
        memory_map_entry_size as u32,  // memory map entry size
        1,   // memory map version

        0,  // [ memory map entry base addr
        0, // ]
        FRAME_SIZE as u32,  // [ memory map entry length
        0, // ]
        1,  // memory map entry type
        1, // memory map entry reserved
        
        9,  // elf
        elf_size as u32,        
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
    let KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address(kernel_heap_addr, 256);

    let frame_allocator = FrameAllocator::new_test(multiboot_header1, KERNEL_BASIC_HEAP_ALLOCATOR);
    let kernel_start_valid_result = Frame::from_address(0);
    let kernel_end_valid_result = Frame::from_address(50 + 50);

    assert!(frame_allocator.kernel_start_frame() == kernel_start_valid_result,
        "Frame allocator failed to determine kernel start frame. Start frame was {}, but should be {}. Frame allocator fields {}",
        frame_allocator.kernel_start_frame(),
        kernel_start_valid_result,
        frame_allocator);

    assert!(frame_allocator.kernel_end_frame() == kernel_end_valid_result,
        "Frame allocator failed to determine kernel end frame. End frame was {}, but should be {}. Frame allocator fields {}",
        frame_allocator.kernel_end_frame(),
        kernel_end_valid_result,
        frame_allocator);
}

#[test]
fn should_make_allocation_in_current_memory_area() {
    let elf_section_entry_size = mem::size_of::<elf::ElfSectionHeader>();
    let elf_size = 5 * mem::size_of::<u32>() + elf_section_entry_size;
    let memory_map_entry_size = mem::size_of::<memory_map::MemoryMapEntry>();
    let memory_map_size = 4 * mem::size_of::<u32>() + 2 * memory_map_entry_size;
    let multiboot_size = 2 * mem::size_of::<u32>() + memory_map_size + elf_size;

    let bytes : [u32; 41] = [
        multiboot_size as u32,          // multiboot length
        1,  // multiboot reserved

        6,  // memory map type
        memory_map_size as u32,         // memory map size
        memory_map_entry_size as u32,  // memory map entry size
        1,   // memory map version

        0,  // [ memory map entry base addr
        0, // ]
        10000,  // [ memory map entry length
        0, // ]
        1,  // memory map entry type
        1, // memory map entry reserved

        20000,  
        0, 
        10000,
        0, 
        1,  
        1, 
                
        9,  // elf
        elf_size as u32,
        1,   // entries num
        elf_section_entry_size as u32,
        1,  //shndx

        1,  //name
        1,  //section type
        0,  //[ flags ]
        0,  //
        2000000,  //[ address ] // somewhere outside testing area
        0,  //
        0,  // [ offset ]
        0,  //
        1000000,  // [ size ]
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
        let KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address(kernel_heap_addr, 256);
        let mut frame_allocator = FrameAllocator::new_test(multiboot_header1, KERNEL_BASIC_HEAP_ALLOCATOR);
        let allocation_result1 = frame_allocator.allocate();
        let allocation_result2 = frame_allocator.allocate();

        
        assert!(allocation_result1.is_some(),
            "Failed first allocation at address 10000. Frame allocator fields {}",
            frame_allocator);

        assert!(allocation_result2.is_some(), 
            "Failed second allocation at address 14096. Frame allocator fields {}",
            frame_allocator);

        let result1 = Frame::from_address(0);
        let result2 = Frame::from_address(0 + FRAME_SIZE);

        assert!(allocation_result1.unwrap() == result1,
            "Invalid returned frame for allocation starting at address 10000. Returned frame {}, but should be {}. Frame allocator fields {}",
            allocation_result1.unwrap(),
            result1,
            frame_allocator);

        assert!(allocation_result2.unwrap() == result2,
            "Invalid returned frame for allocation starting at address 14096. Returned frame {}, but should be {}. Frame allocator fields {}",
            allocation_result1.unwrap(),
            result2,
            frame_allocator);
    };
}

#[test]
fn should_move_to_next_memory_area() {
    let elf_section_entry_size = mem::size_of::<elf::ElfSectionHeader>();
    let elf_size = 5 * mem::size_of::<u32>() + elf_section_entry_size;
    let memory_map_entry_size = mem::size_of::<memory_map::MemoryMapEntry>();
    let memory_map_size = 4 * mem::size_of::<u32>() + 2 * memory_map_entry_size;
    let multiboot_size = 2 * mem::size_of::<u32>() + memory_map_size + elf_size;

    let bytes : [u32; 41] = [
        multiboot_size as u32,          // multiboot length
        1,  // multiboot reserved

        6,  // memory map type
        memory_map_size as u32,         // memory map size
        memory_map_entry_size as u32,  // memory map entry size
        1,   // memory map version

        0,  // [ memory map entry base addr
        0, // ]
        10000,  // [ memory map entry length
        0, // ]
        1,  // memory map entry type
        1, // memory map entry reserved

        10000,
        0, 
        10000,
        0, 
        1,  
        1, 
                
        9,  // elf
        elf_size as u32,
        1,   // entries num
        elf_section_entry_size as u32,
        1,  //shndx

        1,  //name
        1,  //section type
        0,  //[ flags ]
        0,  //
        2000000,  //[ address ] //somewhere outside testing zone
        0,  //
        0,  // [ offset ]
        0,  //
        1000000,  // [ size ]
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
        let KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address(kernel_heap_addr, 256);
        
        let mut frame_allocator = FrameAllocator::new_test(multiboot_header1, KERNEL_BASIC_HEAP_ALLOCATOR);        

        frame_allocator.allocate(); //frame at 00000 - 04905
        frame_allocator.allocate(); // 04906 - 08191
        let allocation_result = frame_allocator.allocate(); // 10000 - 14095, next memory area         
        
        assert!(allocation_result.is_some(),
            "Failed first allocation at address 14096. Frame allocator fields {}",
            frame_allocator);        

        let result1 = Frame::from_address(14096);        

        assert!(allocation_result.unwrap() == result1,
            "Invalid returned frame for allocation starting at address 14096. Returned frame {}, but should be {}. Frame allocator fields {}",
            allocation_result.unwrap(),
            result1,
            frame_allocator);
    };    
}

#[test]
fn should_return_last_freed_frame() {
    let elf_section_entry_size = mem::size_of::<elf::ElfSectionHeader>();
    let elf_size = 5 * mem::size_of::<u32>() + elf_section_entry_size;
    let memory_map_entry_size = mem::size_of::<memory_map::MemoryMapEntry>();
    let memory_map_size = 4 * mem::size_of::<u32>() + 2 * memory_map_entry_size;
    let multiboot_size = 2 * mem::size_of::<u32>() + memory_map_size + elf_size;

    let bytes : [u32; 41] = [
        multiboot_size as u32,          // multiboot length
        1,  // multiboot reserved

        6,  // memory map type
        memory_map_size as u32,         // memory map size
        memory_map_entry_size as u32,  // memory map entry size
        1,   // memory map version

        0,  // [ memory map entry base addr
        0, // ]
        10000,  // [ memory map entry length
        0, // ]
        1,  // memory map entry type
        1, // memory map entry reserved

        10000,
        0, 
        10000,
        0, 
        1,  
        1, 
                
        9,  // elf
        elf_size as u32,
        1,   // entries num
        elf_section_entry_size as u32,
        1,  //shndx

        1,  //name
        1,  //section type
        0,  //[ flags ]
        0,  //
        2000000,  //[ address ] //somewhere outside testing zone
        0,  //
        0,  // [ offset ]
        0,  //
        1000000,  // [ size ]
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
        let KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address(kernel_heap_addr, 256);
        
        let mut frame_allocator = FrameAllocator::new_test(multiboot_header1, KERNEL_BASIC_HEAP_ALLOCATOR);        

        let result = frame_allocator.allocate(); //frame at 00000 - 04905
        
        assert!(result.is_some(),
            "Failed first allocation at address 0. Frame allocator fields {}",
            frame_allocator);

        frame_allocator.deallocate(result.unwrap());
        let result_again = frame_allocator.allocate();
        
        assert!(result_again.is_some(),
            "Failed allocating previously freed frame number 0. Frame allocator fields {}",
            frame_allocator);          

        assert!(result.unwrap() == result_again.unwrap(),
            "Allocated frame doesn't match with latest available free frame. Returned frame {}, but should be {} .Frame allocator fields {}",
            result_again.unwrap(),
            result.unwrap(),
            frame_allocator);
    };    
}