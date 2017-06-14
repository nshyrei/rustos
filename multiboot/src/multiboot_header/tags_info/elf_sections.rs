use stdx::conversion::FromUnsafe;
use core::ptr::read;

#[repr(packed)] // repr(C) would add unwanted padding before first_section
pub struct ElfSections {
    tag_type: u32,
    tag_size: u32,
    entries_num: u32,
    entry_size: u32,
    shndx: u32,
    first_entry: ElfSectionHeader,
}

impl FromUnsafe<usize> for ElfSections {
    unsafe fn from_unsafe(address: usize) -> ElfSections {
        read(address as *const ElfSections)
    }
}

#[repr(C)]
pub struct ElfSectionHeader {
    name: u32,
    pub section_type: u32,
    flags: u64,
    address: usize,
    offset: usize,
    link: u32,
    info: u32,
    address_align: u64,
    entry_size: u64,
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