use multiboot_header::tags_info::elf_sections_iterator::ElfSectionsIterator;
use multiboot_header::multiboot_header_tag::MultibootHeaderTag;

#[derive(Debug)]
#[repr(packed)] // repr(C) would add unwanted padding before first_section
pub struct ElfSections {
    tag_type: u32,
    tag_size: u32,
    entries_num: u32,
    entry_size: u32,
    shndx: u32,
    first_entry: ElfSectionHeader,
}

impl MultibootHeaderTag for ElfSections {
    fn numeric_type() -> u32 {
        9
    }
}

impl ElfSections {
    pub fn entries(&self) -> ElfSectionsIterator {
        let entry_address = (&self.first_entry as *const _ as usize);
        let tag_end_address = (self as *const _ as usize) + self.tag_size as usize;
        ElfSectionsIterator::new(entry_address, tag_end_address, self.entry_size as usize)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ElfSectionHeader {
    name: u32,
    section_type: u32,
    flags: u64,
    address: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    address_align: u64,
    entry_size: u64,
}

impl ElfSectionHeader {
    pub fn section_type(&self) -> u32 {
        self.section_type
    }

    pub fn address(&self) -> u64 {
        self.address
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

#[repr(u32)]
pub enum ElfSectionType {
    Unused = 0,
    ProgramSection = 1,
    LinkerSymbolTable = 2,
    StringTable = 3,
    RelaRelocation = 4,
    SymbolHashTable = 5,
    DynamicLinkingTable = 6,
    Note = 7,
    Uninitialized = 8,
    RelRelocation = 9,
    Reserved = 10,
    DynamicLoaderSymbolTable = 11,
    // plus environment-specific use from 0x60000000 to 0x6FFFFFFF
    // plus processor-specific use from 0x70000000 to 0x7FFFFFFF
}