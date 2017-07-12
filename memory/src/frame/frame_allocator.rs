use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags_info::tag_entry_iterator::TagEntryIterator;
use multiboot::multiboot_header::tags_info::memory_map::*;
use multiboot::multiboot_header::tags_info::elf_sections::ElfSections;
use frame::*;
use kernel::empty_frame_list::EmptyFrameList;
use kernel::frame_bitmap::FrameBitMap;
use kernel::bump_allocator::BumpAllocator;

pub struct FrameAllocator {
    multiboot_start_frame: Frame,
    multiboot_end_frame: Frame,
    kernel_start_frame: Frame,
    kernel_end_frame: Frame,
    current_memory_area : &'static MemoryMapEntry,
    memory_areas: AvailableMemorySectionsIterator,
    last_frame_number: usize,
    frame_bit_map: &'static FrameBitMap,
    empty_frame_list: &'static EmptyFrameList,
}

impl FrameAllocator {
    pub fn new(multiboot_header: &'static MultibootHeader, kernel_heap_allocator : &'static mut BumpAllocator) -> FrameAllocator {
        let elf_sections = multiboot_header.read_tag::<ElfSections>();
        let memory_areas = multiboot_header.read_tag::<MemoryMap>();

        let mut available_memory_areas = memory_areas.entries();

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
        let last_frame_number = Frame::from_address(first_memory_area.base_address() as usize).number();

        FrameAllocator {
            multiboot_start_frame: Frame::from_address(multiboot_header.start_address()),
            multiboot_end_frame: Frame::from_address(multiboot_header.end_address()),
            kernel_start_frame: Frame::from_address(kernel_start_address),
            kernel_end_frame: Frame::from_address(kernel_end_address),
            current_memory_area : first_memory_area,
            memory_areas: available_memory_areas,
            last_frame_number: last_frame_number,
            frame_bit_map: FrameBitMap::new(available_memory, Frame_Size, kernel_heap_allocator),
            empty_frame_list: EmptyFrameList::new(0, kernel_heap_allocator),
        }
    }

    pub fn allocate(&mut self) -> Frame {
        // try return last_frame_number

        let result = self.step_over_reserved_memory_if_needed();                
        let last_frame_in_area = Frame::from_address(self.current_memory_area.end_address() as usize);

        if(result > last_frame_in_area.number()){
            self.next_memory_area(result);
            self.allocate()
        } else {
            self.frame_bit_map.set_in_use(result);
            self.last_frame_number = result + 1;
            Frame::new(result)
        }        
    }

    fn step_over_reserved_memory_if_needed(&self) -> usize {
        if(self.last_frame_number >= self.multiboot_start_frame.number() &&
           self.last_frame_number <= self.multiboot_end_frame.number()){
            self.multiboot_end_frame.number() + 1
        }
        else if(self.last_frame_number >= self.kernel_start_frame.number() &&
                self.last_frame_number <= self.kernel_end_frame.number()){
            self.kernel_end_frame.number() + 1
        }
        else{
            self.last_frame_number
        }
    }

    fn next_memory_area(&mut self, last_frame_number : usize) {
        // find memory area with lowest base addr and which can hold provided frame
        let new_current_memory_area = self.memory_areas
            .clone()
            .filter(|e| Frame::from_address(e.base_address() as usize).number() >= last_frame_number)
            .min_by_key(|e| e.base_address())
            .unwrap();

        self.current_memory_area = new_current_memory_area;
        self.last_frame_number = Frame::from_address(new_current_memory_area.base_address() as usize).number();
    }

    pub fn deallocate(&mut self, frame : Frame) {

    }
}