use stdx::conversion::FromUnsafe;

pub trait MultiBootTagInfo {}


pub struct BasicMemoryInfo {
    pub memory_lower: u32,
    pub memory_upper: u32,
}

impl FromUnsafe<usize> for BasicMemoryInfo {
    unsafe fn from_unsafe(address: usize) -> BasicMemoryInfo {
        let (_, _, memory_lower, memory_upper) = *(address as *const (u32, u32, u32, u32));
        BasicMemoryInfo {
            memory_lower: memory_lower,
            memory_upper: memory_upper,
        }
    }
}

impl MultiBootTagInfo for BasicMemoryInfo {}

pub struct MemoryMap {
    pub version: u32,
    pub entries: [MemoryMapEntry; 10], //change to dynamic array/collection
}

impl FromUnsafe<usize> for MemoryMap {
    unsafe fn from_unsafe(address: usize) -> MemoryMap {
        let (_, tag_size, entry_size, version) = *(address as *const (u32, u32, u32, u32));
        let mut entry_address = address + 16; //4 u32 fields offset
        let mut entry = *(entry_address as *const MemoryMapEntry);
        let mut memory_entries: [MemoryMapEntry; 10] = [MemoryMapEntry::default(); 10];
        memory_entries[0] = entry;

        let mut entry_counter = 1;
        while entry_address < address + tag_size as usize {
            entry_address += entry_size as usize;
            entry = *(entry_address as *const MemoryMapEntry);
            memory_entries[entry_counter] = entry;

            entry_counter += 1;
        }

        MemoryMap {
            version: version,
            entries: memory_entries,
        }
    }
}

impl MultiBootTagInfo for MemoryMap {}

#[derive(Copy, Clone)]
pub struct MemoryMapEntry {
    pub base_address: u64,
    pub length: u64,
    pub entry_type: u32,
    reserved: u32,
}

impl MemoryMapEntry {
    pub fn default() -> MemoryMapEntry {
        MemoryMapEntry {
            base_address: 0,
            length: 0,
            entry_type: 0,
            reserved: 0,
        }
    }
}

pub struct ElfSections {
    pub entries_num: u32,
    pub entry_size: u32,
    pub shndx: u32,
    pub entries: [ElfSectionHeader; 10], //change to dynamic array/collection
}

impl FromUnsafe<usize> for ElfSections {
    unsafe fn from_unsafe(address: usize) -> ElfSections {
        let (_, tag_size, entry_number, entry_size, shndx) = *(address as
                                                               *const (u32, u32, u32, u32, u32));
        let mut entry_address = address + 20;
        let mut entry = *(entry_address as *const ElfSectionHeader);
        let mut memory_entries: [ElfSectionHeader; 10] = [ElfSectionHeader::default(); 10];
        memory_entries[0] = entry;

        let mut entry_counter = 1;
        while entry_address < address + tag_size as usize {
            entry_address += entry_size as usize;
            entry = *(entry_address as *const ElfSectionHeader);
            memory_entries[entry_counter] = entry;

            entry_counter += 1;
        }

        ElfSections {
            entries_num: entry_number,
            entry_size: entry_size,
            shndx: shndx,
            entries: memory_entries,
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
