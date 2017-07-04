use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags_info::tag_entry_iterator::TagEntryIterator;
use multiboot::multiboot_header::tags_info::memory_map::{MemoryMapEntry, MemoryMap};
use multiboot::multiboot_header::tags_info::elf_sections::ElfSections;

pub struct FrameAllocator {
    multiboot_start_address: usize,
    multiboot_end_address: usize,
    kernel_start_address: usize,
    kernel_end_address: usize,
    memory_areas: TagEntryIterator<MemoryMapEntry>,
    last_free_frame: u32,
}

impl FrameAllocator {
    pub fn new(multiboot_header: &'static MultibootHeader) -> FrameAllocator {
        let elf_sections = multiboot_header.read_tag::<ElfSections>();
        let memory_areas_it = multiboot_header.read_tag::<MemoryMap>().entries();
        let kernel_start_section = elf_sections.entries().min_by_key(|e| e.address()).unwrap();
        let kernel_end_section = elf_sections
            .entries()
            .max_by_key(|e| e.address() + e.size())
            .unwrap();

        let kernel_start_address = kernel_start_section.address() as usize;
        let kernel_end_address = kernel_end_section.address() as usize;


        FrameAllocator {
            multiboot_start_address: multiboot_header.start_address(),
            multiboot_end_address: multiboot_header.end_address(),
            kernel_start_address: kernel_start_address,
            kernel_end_address: kernel_end_address,
            memory_areas: memory_areas_it,
            last_free_frame: 0,
        }
    }
}