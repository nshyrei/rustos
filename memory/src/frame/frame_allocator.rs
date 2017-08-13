use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags_info::memory_map::*;
use multiboot::multiboot_header::tags_info::elf_sections::ElfSections;
use frame::*;
use kernel::empty_frame_list::EmptyFrameList;
use kernel::frame_bitmap::FrameBitMap;
use core::fmt;
use stdx::util;
use kernel::bump_allocator::BumpAllocator;
/*
    Bump allocator with a stack of free frames. Allocates new frames in simple incremental fashion
    not touching multiboot and kernel code. On deallocate places frame into free frame stack and picks
    frames from it on further allocations if possible. Bump allocation stops entirely when there is no more
    available memory (allocation try is outside available memory range) and stack of free frames is used after that. 
    You can think of bump allocation as a meachanism to populate free frame stack, the other option to do it without bump allocation is to 
    create a big free frame stack that describes all available memory and only use it.  
*/
pub struct FrameAllocator {
    multiboot_start_frame: Frame,
    multiboot_end_frame: Frame,
    kernel_start_frame: Frame,
    kernel_end_frame: Frame,
    current_memory_area : &'static MemoryMapEntry,
    memory_areas: AvailableMemorySectionsIterator,
    last_frame_number: Frame,
    frame_bit_map: FrameBitMap,
    empty_frame_list: util::Option<&'static EmptyFrameList>,
    KERNEL_BASIC_HEAP_ALLOCATOR : &'static mut BumpAllocator
}

impl FrameAllocator {

    pub fn multiboot_start_frame(&self) -> Frame {
        self.multiboot_start_frame
    }

    pub fn multiboot_end_frame(&self) -> Frame {
        self.multiboot_end_frame
    }

    pub fn kernel_start_frame(&self) -> Frame {
        self.kernel_start_frame
    }

    pub fn kernel_end_frame(&self) -> Frame {
        self.kernel_end_frame
    }

    pub fn current_memory_area(&self) -> &'static MemoryMapEntry {
        self.current_memory_area
    }

    pub fn memory_areas(&self) -> AvailableMemorySectionsIterator {
        self.memory_areas.clone()
    }

    pub fn last_frame_number(&self) -> Frame {
        self.last_frame_number
    }    

    pub fn empty_frame_list(&self) -> util::Option<&'static EmptyFrameList> {
        self.empty_frame_list
    }

    pub fn bit_map_size(&self) -> usize {
        self.frame_bit_map.size()
    }
    
    pub fn new(multiboot_header: &'static MultibootHeader, KERNEL_BASIC_HEAP_ALLOCATOR : &'static mut BumpAllocator) -> FrameAllocator {
        let elf_sections = multiboot_header.read_tag::<ElfSections>()
            .expect("Cannot create frame allocator without multiboot elf sections");
        let memory_areas = multiboot_header.read_tag::<MemoryMap>()
            .expect("Cannot create frame allocator without multiboot memory map");        

        if elf_sections.entries().count() == 0 {
            panic!("No elf sections, cannot determine kernel code address");
        }

        if memory_areas.entries().count() == 0 {
            panic!("No available memory areas for frame allocator");
        }
                
        let kernel_start_section = elf_sections.entries()
            .min_by_key(|e| e.address())
            .unwrap();
        let kernel_end_section = elf_sections.entries()
            .max_by_key(|e| e.end_address())
            .unwrap();

        let kernel_start_address = kernel_start_section.address() as usize;
        let kernel_end_address = kernel_end_section.end_address() as usize;

        let available_memory = memory_areas
            .entries()
            .fold(0, |base, e| base + e.length()) as usize;

        let first_memory_area = memory_areas.entries()
            .min_by_key(|e| e.base_address())
            .unwrap();

        let last_frame_number = Frame::from_address(first_memory_area.base_address() as usize);

        let start_free_list = EmptyFrameList::new_tail(last_frame_number, KERNEL_BASIC_HEAP_ALLOCATOR);

        FrameAllocator {
            multiboot_start_frame: Frame::from_address(multiboot_header.start_address()),
            multiboot_end_frame: Frame::from_address(multiboot_header.end_address()),
            kernel_start_frame: Frame::from_address(kernel_start_address),
            kernel_end_frame: Frame::from_address(kernel_end_address),
            current_memory_area : first_memory_area,
            memory_areas: memory_areas.entries(),
            last_frame_number: last_frame_number,
            frame_bit_map: FrameBitMap::new(available_memory, FRAME_SIZE, KERNEL_BASIC_HEAP_ALLOCATOR),
            empty_frame_list: util::Option(Some(start_free_list)),
            KERNEL_BASIC_HEAP_ALLOCATOR : KERNEL_BASIC_HEAP_ALLOCATOR
        }
    }
    
    pub fn allocate(&mut self) -> Option<Frame> {
        
        // check empty frame list first, if nothing perform bump allocation        
        if let Some(empty_frame_list) = self.empty_frame_list.0 {
            // pick first result from empty frame list
            let (result, tail) = empty_frame_list.take(self.KERNEL_BASIC_HEAP_ALLOCATOR);

            self.empty_frame_list = util::Option(tail);
            self.frame_bit_map.set_in_use(result.number());

            Some(result)
        }
        else {
            match self.bump_allocate() {
                Some(allocate_result) => {                    
                    self.last_frame_number = allocate_result.next(); // next possible frame for bump allocator
                    self.frame_bit_map.set_in_use(allocate_result.number());

                    Some(allocate_result)
                }
                // if we can't allocate with bump and there is nothing in free frame stack
                // then we are out of memory
                None => None
            }
        }
    }

    /*
        Returns next free frame in a form of EmptyFrameList cell that has no self.next (bottom of the stack). 
        Changes self.current_memory_area when moving to new memory area.
    */
    fn bump_allocate(&mut self) -> Option<Frame> {
        let result = self.step_over_reserved_memory_if_needed();        

        if result.end_address() > self.current_memory_area.end_address() as usize {  
            if let Some(memory_area) = self.next_fitting_memory_area(result) {
                self.current_memory_area = memory_area;

                let result = Frame::from_address(memory_area.base_address() as usize);
                Some(result)
            } else {
                None
            }            
        } else {
            Some(result)
        }
    }

    fn step_over_reserved_memory_if_needed(&self) -> Frame {
        // dont touch multiboot data
        if self.last_frame_number >= self.multiboot_start_frame &&
           self.last_frame_number <= self.multiboot_end_frame {
            self.multiboot_end_frame.next()
        }
        // dont touch kernel code
        else if self.last_frame_number >= self.kernel_start_frame &&
                self.last_frame_number <= self.kernel_end_frame {
            self.kernel_end_frame.next()
        }
        else{
            self.last_frame_number
        }
    }

    /*
        Tries to find memory area with lowest base addr and which can hold provided frame
    */
    fn next_fitting_memory_area(&self, last_frame_number : Frame) -> Option<&'static MemoryMapEntry> {        
        self.memory_areas
            .clone()
            .filter(|e| Frame::from_address(e.base_address() as usize) >= last_frame_number)
            .min_by_key(|e| e.base_address())            
    }

    pub fn deallocate(&mut self, frame : Frame) {
        self.frame_bit_map.set_free(frame.number());
        let new_empty_frame_list = self.empty_frame_list.0.map_or(EmptyFrameList::new_tail(frame, self.KERNEL_BASIC_HEAP_ALLOCATOR), |e| e.add(frame, self.KERNEL_BASIC_HEAP_ALLOCATOR));

        self.empty_frame_list = util::Option(Some(new_empty_frame_list));
    }
}

impl fmt::Display for FrameAllocator {    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "multiboot_start_frame: {}, 
               multiboot_end_frame : {},
               kernel_start_frame : {},
               kernel_end_frame : {},
               current_memory_area : {},
               memory_areas : {},
               last_frame_number : {},
               frame_bit_map : {},
               empty_frame_list : {}",
               self.multiboot_start_frame,
               self.multiboot_end_frame,
               self.kernel_start_frame,
               self.kernel_end_frame,
               self.current_memory_area,
               self.memory_areas,
               self.last_frame_number,
               self.frame_bit_map,
               self.empty_frame_list)
    }
}