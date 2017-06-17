use stdx::conversion::FromUnsafe;
use core::ptr::read;

#[derive(Debug)]
#[repr(packed)] // repr(C) would add unwanted padding before first_section
pub struct ElfSectionsTag {
    typ: u32,
    size: u32,
    pub number_of_sections: u32,
    entry_size: u32,
    shndx: u32, // string table
    first_section: ElfSection,
}


impl FromUnsafe<usize> for ElfSectionsTag {
    unsafe fn from_unsafe(address: usize) -> ElfSectionsTag {
        read(address as *const ElfSectionsTag)
    }
}

impl ElfSectionsTag {
    pub unsafe fn from_unsafe1(address: usize) -> &'static ElfSectionsTag {
        &(*(address as *const ElfSectionsTag))
    }

    pub fn sections(self) -> ElfSectionIter {

        let rrr = (&self.first_section as *const _ as u64);
        let slf = (&self as *const _ as u64);
        ElfSectionIter {
            current_section: self.first_section,
            remaining_sections: self.number_of_sections - 1,
            entry_size: self.entry_size,
        }
    }
    pub fn sections1(&self) -> ElfSectionIter {

        let slf = (self as *const _ as u64);
        ElfSectionIter {
            current_section: self.first_section,
            remaining_sections: self.number_of_sections - 1,
            entry_size: self.entry_size,
        }
    }
}

#[derive(Clone)]
pub struct ElfSectionIter {
    current_section: ElfSection,
    remaining_sections: u32,
    entry_size: u32,
}

impl Iterator for ElfSectionIter {
    type Item = ElfSection;
    fn next(&mut self) -> Option<ElfSection> {
        if self.remaining_sections == 0 {
            None
        } else {
            let section = self.current_section;
            let curerntAddr = (&self.current_section as *const _ as u64);
            let next_section_addr = (&self.current_section as *const _ as u64) +
                                    self.entry_size as u64;
            self.current_section = unsafe { *(next_section_addr as *const ElfSection) };
            self.remaining_sections -= 1;
            if section.typ == ElfSectionType::Unused as u32 {
                self.next()
            } else {
                Some(section)
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ElfSection {
    name_index: u32,
    pub typ: u32,
    pub flags: u64,
    pub addr: u64,
    offset: u64,
    pub size: u64,
    link: u32,
    info: u32,
    addralign: u64,
    entry_size: u64,
}

impl ElfSection {
    pub fn start_address(&self) -> usize {
        self.addr as usize
    }

    pub fn end_address(&self) -> usize {
        (self.addr + self.size) as usize
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