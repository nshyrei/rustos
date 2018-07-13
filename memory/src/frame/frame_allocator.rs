use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags::memory_map::*;
use multiboot::multiboot_header::tags::elf;
use frame::Frame;
use frame::FRAME_SIZE;
use stdx_memory::collections::linked_list::LinkedList;
use allocator::bump::{BumpAllocator, ConstSizeBumpAllocator};
use stdx_memory::MemoryAllocator;
use stdx_memory::heap;
use core::fmt;
use core::mem;
use core::ptr;


/*
    Bump allocator with a stack of free frames. Allocates new frames in simple incremental fashion
    not touching multiboot and kernel code. On deallocate places frame into free frame stack and picks
    frames from it on further allocations if possible. Bump allocation stops entirely when there is no more
    available memory (allocation try is outside available memory range) and stack of free frames is used after that. 
    You can think of bump allocation as a meachanism to populate free frame stack, the other option to do it without bump allocation is to 
    create a big free frame stack that describes all available memory and use only it.  
*/
pub struct FrameAllocator {
    multiboot_start_frame: Frame,
    multiboot_end_frame: Frame,
    kernel_start_frame: Frame,
    kernel_end_frame: Frame,
    current_memory_area : ptr::NonNull<MemoryMapEntry>,
    memory_areas: AvailableMemorySectionsIterator,
    last_frame_number: Frame,
    empty_frame_list: heap::Box<LinkedList<Frame>>,
    frame_list_allocator : ConstSizeBumpAllocator,
    buddy_allocator_start_frame : Frame,
    buddy_allocator_end_frame : Frame
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

    pub fn current_memory_area(&self) -> &MemoryMapEntry {
        unsafe {
        self.current_memory_area.as_ref()
        }
    }

    pub fn memory_areas(&self) -> AvailableMemorySectionsIterator {
        self.memory_areas.clone()
    }

    pub fn last_frame_number(&self) -> Frame {
        self.last_frame_number
    }

    pub fn bump_allocator(&self) -> ConstSizeBumpAllocator {
        self.frame_list_allocator.clone()
    }

    pub fn end_address(&self) -> usize {
        self.frame_list_allocator.end_address()
    }
        
    pub fn set_buddy_start(&mut self, f : Frame){
        self.buddy_allocator_start_frame = f
    }

    pub fn set_buddy_end(&mut self, f : Frame){
        self.buddy_allocator_end_frame = f
    }

    fn empty_frame_list(&self) -> &heap::Box<LinkedList<Frame>> {
        &self.empty_frame_list
    }

    pub fn new_test(multiboot_header: &MultibootHeader, bump_allocator1 : ConstSizeBumpAllocator) -> FrameAllocator {
        let elf_sections = multiboot_header.read_tag::<elf::ElfSections>()
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
            .min_by_key(|e| e.start_address())
            .unwrap();
        let kernel_end_section = elf_sections.entries()
            .max_by_key(|e| e.end_address())
            .unwrap();

        let kernel_start_address = kernel_start_section.start_address() as usize;
        let kernel_end_address = kernel_end_section.end_address() as usize;        
            
        let first_memory_area = FrameAllocator::next_fitting_memory_area(memory_areas.entries(), Frame::from_address(0)).expect("Cannot determine first memory area");            
        let last_frame_number = FrameAllocator::frame_for_base_address(first_memory_area.base_address() as usize);        
        let mut bump_allocator = bump_allocator1;

        FrameAllocator {
            multiboot_start_frame: Frame::from_address(multiboot_header.start_address()),
            multiboot_end_frame: Frame::from_address(multiboot_header.end_address()),
            kernel_start_frame: Frame::from_address(kernel_start_address),
            kernel_end_frame: Frame::from_address(kernel_end_address),
            current_memory_area : ptr::NonNull::from(first_memory_area),
            memory_areas: memory_areas.entries(),
            last_frame_number: last_frame_number,
            empty_frame_list: heap::Box::new(LinkedList::Nil, &mut bump_allocator),
            frame_list_allocator : bump_allocator,
            buddy_allocator_start_frame : Frame::from_address(0),
            buddy_allocator_end_frame : Frame::from_address(0)
        }
    }


    pub fn new(multiboot_header: &MultibootHeader) -> FrameAllocator {
        let elf_sections = multiboot_header.read_tag::<elf::ElfSections>()
            .expect("Cannot create frame allocator without multiboot elf sections");
        let memory_areas = multiboot_header.read_tag::<MemoryMap>()
            .expect("Cannot create frame allocator without multiboot memory map");        

        assert!(elf_sections.entries().count() != 0, "No elf sections, cannot determine kernel code address");        
        assert!(memory_areas.entries().count() != 0, "No available memory areas for frame allocator");
        
        let kernel_start_address = elf_sections.entries_start_address().unwrap() as usize;
        let kernel_end_address = elf_sections.entries_end_address().unwrap() as usize;        
            
        let first_memory_area = FrameAllocator::next_fitting_memory_area(memory_areas.entries(), Frame::from_address(0)).expect("Cannot determine first memory area");            
        let last_frame_number = FrameAllocator::frame_for_base_address(first_memory_area.base_address() as usize);        

        let empty_frame_list_size = FrameAllocator::get_empty_frame_list_size(&memory_areas);
        let kernel_end_frame = Frame::from_address(kernel_end_address);        
        // move it to some proper place!
        let mut bump_allocator = ConstSizeBumpAllocator::from_address_for_type::<LinkedList<Frame>>(multiboot_header.end_address() + 1, empty_frame_list_size);

        FrameAllocator {
            multiboot_start_frame: Frame::from_address(multiboot_header.start_address()),
            multiboot_end_frame: Frame::from_address(multiboot_header.end_address()),
            kernel_start_frame: Frame::from_address(kernel_start_address),
            kernel_end_frame: kernel_end_frame,
            current_memory_area : ptr::NonNull::from(first_memory_area),
            memory_areas: memory_areas.entries(),
            last_frame_number: last_frame_number,
            empty_frame_list: heap::Box::new(LinkedList::Nil, &mut bump_allocator),
            frame_list_allocator : bump_allocator,
            buddy_allocator_start_frame : Frame::from_address(0),
            buddy_allocator_end_frame : Frame::from_address(0)
        }
    }
    
    fn get_empty_frame_list_size(memory_map : &MemoryMap) -> usize {
        let available_memory = memory_map.available_memory() as usize;
        let total_frames_count = available_memory / FRAME_SIZE;

        total_frames_count * mem::size_of::<LinkedList<Frame>>()
    }

    pub fn allocate(&mut self) -> Option<Frame> {
                        
        // check empty frame list first, if nothing perform bump allocation
        match self.empty_frame_list.take() {
            Some((value, prev)) => {
                
                // pick first result from empty frame list                
                self.empty_frame_list = prev;                
                
                Some(value)
            },
            _ => {
                match self.bump_allocate() {
                    Some(allocate_result) => {                    
                        self.last_frame_number = allocate_result.next(); // next possible frame for bump allocator
                        //self.frame_bit_map.set_in_use(allocate_result.number());

                        Some(allocate_result)
                    },
                    // if we can't allocate with bump and there is nothing in free frame stack
                    // then we are out of memory
                    None => None
                }
            }
        }
    }

    /*
        tries to return self.last_frame_number first, if it fails tries to return self.last_frame_number.next
        Changes self.current_memory_area when moving to new memory area.
    */
    fn bump_allocate(&mut self) -> Option<Frame> {
        unsafe {
        let possible_frame = self.last_frame_number;
        let result = self.step_over_reserved_memory_if_needed(possible_frame);        

        if result.end_address() > self.current_memory_area.as_ref().end_address() as usize {  
            if let Some(memory_area) = FrameAllocator::next_fitting_memory_area(self.memory_areas.clone(), result) {
                self.current_memory_area = ptr::NonNull::from(memory_area);

                let result = FrameAllocator::frame_for_base_address(memory_area.base_address() as usize);
                Some(result)
            } else {
                None
            }            
        } else {
            Some(result)
        }
        }
    }

    // base address of memory area must be frame aligned or it will result in bugs like this:
    // For example we have memory area starting at 1000
    // and frame size iz 4000, thus creating frame from address 1000 will result in frame
    // with number = 0, which has base address = 0 also, which is below memory area and will
    // result in memory read fault
    fn frame_for_base_address(base_address : usize) -> Frame {
        let first_attempt_frame = Frame::from_address(base_address);

        if Frame::is_frame_aligned(base_address) {
            first_attempt_frame
        }
        else {
            first_attempt_frame.next()
        }
    }

    fn step_over_reserved_memory_if_needed(&self, frame : Frame) -> Frame {
        // dont touch multiboot data
        if frame >= self.multiboot_start_frame &&
           frame <= self.multiboot_end_frame {
            self.step_over_reserved_memory_if_needed(self.multiboot_end_frame.next()) // in case next will touch kernel code
        }
        // dont touch kernel code
        else if frame >= self.kernel_start_frame &&
                frame <= self.kernel_end_frame {
            self.step_over_reserved_memory_if_needed(self.kernel_end_frame.next()) // in case next will touch empty frame list
        }
        // dont touch empty frame list
        else if frame >= Frame::from_address(self.frame_list_allocator.start_address()) &&
                frame <= Frame::from_address(self.frame_list_allocator.end_address()) {
            let possible_frame = Frame::from_address(self.frame_list_allocator.end_address()).next();
            self.step_over_reserved_memory_if_needed(possible_frame) // in case next() will touch heap data structure
        }
        // don't touch heap
        else if frame >= self.buddy_allocator_start_frame &&
                frame <= self.buddy_allocator_end_frame
        {
            let possible_frame = self.buddy_allocator_end_frame.next();
            self.step_over_reserved_memory_if_needed(possible_frame) // in case next() will touch heap data structure
        }
        else{
            frame
        }
    }

    /*
        Try to find memory area with lowest base addr and which can hold provided frame
    */
    fn next_fitting_memory_area(memory_areas : AvailableMemorySectionsIterator, last_frame_number : Frame) -> Option<&'static MemoryMapEntry> {        
        memory_areas
            .clone()       
            .filter(|e| {
                let frame = FrameAllocator::frame_for_base_address(e.base_address() as usize);
                // frame must be fully inside memory area
                frame.end_address() <= e.end_address() as usize && frame >= last_frame_number
             })
            .min_by_key(|e| e.base_address())
    }

    pub fn deallocate(&mut self, frame : Frame) {   
        /*     
        let new_list = self.empty_frame_list.map_or(
            FreeList::new(frame, &mut self.frame_list_allocator),
            |e| e.pointer().add(frame, &mut self.frame_list_allocator)          
        );

        self.empty_frame_list = Some(new_list);
        */
    }
}

impl fmt::Display for FrameAllocator {    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
        write!(f,
               "multiboot_start_frame: {}, 
               multiboot_end_frame : {},
               kernel_start_frame : {},
               kernel_end_frame : {},
               current_memory_area : {},
               memory_areas : {},
               last_frame_number : {},               
               empty_frame_list : {}",
               self.multiboot_start_frame,
               self.multiboot_end_frame,
               self.kernel_start_frame,
               self.kernel_end_frame,
               self.current_memory_area.as_ref(),
               self.memory_areas,
               self.last_frame_number,
               //self.frame_bit_map,
               "")
    }
    }
}