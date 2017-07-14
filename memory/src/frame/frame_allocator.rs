use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags_info::memory_map::*;
use multiboot::multiboot_header::tags_info::elf_sections::ElfSections;
use frame::*;
use kernel::empty_frame_list::EmptyFrameList;
use kernel::frame_bitmap::FrameBitMap;
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
    frame_bit_map: &'static FrameBitMap,
    empty_frame_list: Option<&'static EmptyFrameList>
}

impl FrameAllocator {
    pub fn new(multiboot_header: &'static MultibootHeader, kernel_heap_allocator : &'static mut BumpAllocator) -> FrameAllocator {
        let elf_sections = multiboot_header.read_tag::<ElfSections>();
        let memory_areas = multiboot_header.read_tag::<MemoryMap>();

        let available_memory_areas = memory_areas.entries();

        let kernel_start_section = elf_sections
            .entries()
            .min_by_key(|e| e.address())
            .unwrap();

        let kernel_end_section = elf_sections
            .entries()
            .max_by_key(|e| e.address() + e.size())
            .unwrap();

        let kernel_start_address = kernel_start_section.address() as usize;
        let kernel_end_address = kernel_end_section.address() as usize;

        let available_memory = available_memory_areas
            .clone()
            .fold(0, |base, e| base + e.length()) as usize;

        let first_memory_area = available_memory_areas.clone().min_by_key(|e| e.base_address()).unwrap();
        let last_frame_number = Frame::from_address(first_memory_area.base_address() as usize);

        FrameAllocator {
            multiboot_start_frame: Frame::from_address(multiboot_header.start_address()),
            multiboot_end_frame: Frame::from_address(multiboot_header.end_address()),
            kernel_start_frame: Frame::from_address(kernel_start_address),
            kernel_end_frame: Frame::from_address(kernel_end_address),
            current_memory_area : first_memory_area,
            memory_areas: available_memory_areas,
            last_frame_number: last_frame_number,
            frame_bit_map: FrameBitMap::new(available_memory, FRAME_SIZE, kernel_heap_allocator),
            empty_frame_list: None,
        }
    }
    
    pub fn allocate(&mut self) -> Option<Frame> {
        
        if let Some(empty_frame_list) = self.empty_frame_list {
            // pick first result from empty frame list
            let (result, tail) = empty_frame_list.take();

            self.empty_frame_list = tail;
            self.frame_bit_map.set_in_use(result.number());

            Some(result)
        }
        else {
            let allocate_result = self.bump_allocate();
            if allocate_result.is_some() {
                self.empty_frame_list = allocate_result;
                self.last_frame_number = allocate_result // next possible frame for bump allocator
                    .unwrap()
                    .value()
                    .next(); 
                self.allocate()
            }
            else {
                // if we can't allocate with bump and there is nothing in free frame stack
                // then we are out of memory
                None
            }
        }
    }

    /*
        Returns next free frame in a form of EmptyFrameList cell that has no self.next (bottom of the stack). 
        Changes self.current_memory_area when moving to new memory area.
    */
    fn bump_allocate(&mut self) -> Option<&'static EmptyFrameList> {
        let result = self.step_over_reserved_memory_if_needed();                
        let last_frame_in_area = Frame::from_address(self.current_memory_area.end_address() as usize);

        if result > last_frame_in_area {            
            if let Some(memory_area) = self.next_fitting_memory_area(result) {                
                self.current_memory_area = memory_area;

                let result = Frame::from_address(memory_area.base_address() as usize);
                Some(EmptyFrameList::new_tail(result))
            } else {
                None
            }            
        } else {
            Some(EmptyFrameList::new_tail(result))
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
        let new_empty_frame_list = self.empty_frame_list.map_or(EmptyFrameList::new_tail(frame), |e| e.add(frame));

        self.empty_frame_list = Some(new_empty_frame_list);
    }
}