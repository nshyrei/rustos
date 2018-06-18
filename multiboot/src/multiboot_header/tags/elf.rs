use multiboot_header::MultibootHeaderTag;
use core::fmt;
use core::iter;

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

    pub fn entries_start_address(&self) -> Option<u64> {
        self.entries()
            .min_by_key(|e| e.start_address())
            .map(|e| e.start_address())            
    }

    pub fn entries_end_address(&self) -> Option<u64> {
        self.entries()
            .max_by_key(|e| e.end_address())
            .map(|e| e.end_address()) 
    }
}

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

    pub fn start_address(&self) -> u64 {
        self.address
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn end_address(&self) -> u64 {
        self.address + self.size - 1
    }

    pub fn flags(&self) -> ElfSectionFlags {
        ElfSectionFlags::from_bits_truncate(self.flags)
    }
}

impl fmt::Display for ElfSectionHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "name: {},
        section_type: {},
        flags: {},
        address: {},
        offset: {},
        size: {},
        link: {},
        info: {},
        address_align: {},
        entry_size: {}",
               self.name,
               self.section_type,
               self.flags,
               self.address,
               self.offset,
               self.size,
               self.link,
               self.info,
               self.address_align,
               self.entry_size)
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



bitflags! {
    pub struct ElfSectionFlags : u64 {
        const WRITABLE = 0x1;
        const ALLOCATED = 0x2;
        const EXECUTABLE = 0x4;
        // plus environment-specific use at 0x0F000000
        // plus processor-specific use at 0xF0000000
    }
}

pub struct ElfSectionsIterator {
    entry_address: usize,
    tag_end_address: usize,
    entry_size: usize,
}

impl ElfSectionsIterator {
    pub fn new(entry_address: usize, tag_end_address: usize, entry_size: usize) -> ElfSectionsIterator {
        ElfSectionsIterator {
            entry_address: entry_address,
            tag_end_address: tag_end_address,
            entry_size: entry_size,
        }
    }
}

impl iter::Iterator for ElfSectionsIterator {
    type Item = &'static ElfSectionHeader;

    fn next(&mut self) -> Option<&'static ElfSectionHeader> {
        if self.entry_address >= self.tag_end_address {
            None
        } else {
            let result = unsafe { &(*(self.entry_address as *const ElfSectionHeader)) };
            self.entry_address += self.entry_size;
            // skip unused
            // todo possibly replace with loop if this will compile to recursion
            if result.section_type() == ElfSectionType::Unused as u32 {
                self.next()
            } else {
                Some(result)
            }
        }
    }
}