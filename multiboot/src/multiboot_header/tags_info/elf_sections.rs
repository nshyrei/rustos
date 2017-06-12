use stdx::conversion::FromUnsafe;
use multiboot_header::tags_info::tag_entry_iterator::TagEntryIterator;
use core::ptr::read;

pub struct ElfSections {
    pub entries_num: u32,
    pub entry_size: u32,
    pub shndx: u32,
    pub entries: TagEntryIterator<ElfSectionHeader>,
}

impl FromUnsafe<usize> for ElfSections {
    unsafe fn from_unsafe(address: usize) -> ElfSections {
        let (_, tag_size, entry_number, entry_size, shndx) = read(address as *const (u32, u32, u32, u32, u32));
        let entry_address = address + 20;
        let tag_end_address = address + tag_size as usize;

        ElfSections {
            entries_num: entry_number,
            entry_size: entry_size,
            shndx: shndx,
            entries: TagEntryIterator::new(entry_address, tag_end_address, entry_size as usize),
        }
    }
}

#[derive(Copy, Clone)]
pub struct ElfSectionHeader {
    name: u32,
    section_type: u32,
    flags: u64,
    address: usize,
    offset: usize,
    link: u32,
    info: u32,
    address_align: u64,
    entry_size: u64,
}

impl ElfSectionHeader {
    pub fn default() -> ElfSectionHeader {
        ElfSectionHeader {
            name: 0,
            section_type: 0,
            flags: 0,
            address: 0,
            offset: 0,
            link: 0,
            info: 0,
            address_align: 0,
            entry_size: 0,
        }
    }
}
