use multiboot_header::MultibootHeaderTag;
use core::fmt;
use core::iter;

#[repr(C)] // repr(C) is crucial to make read(address as *const MemoryMap) work properly
// default struct pack couldn't be read like this
pub struct MemoryMap {
    tag_type: u32,
    tag_size: u32,
    entry_size: u32,
    version: u32,
    first_entry: MemoryMapEntry,
}

impl MemoryMap {
    pub fn entries(&self) -> AvailableMemorySectionsIterator {
        let entry_address = (&self.first_entry) as *const _ as usize;
        let tag_end_address = (self as *const _ as usize) + self.tag_size as usize;
        AvailableMemorySectionsIterator::new(entry_address, tag_end_address, self.entry_size as usize)
    }
}

impl MultibootHeaderTag for MemoryMap {
    fn numeric_type() -> u32 {
        6
    }
}

#[repr(C)]
pub struct MemoryMapEntry {
    base_address: u64,
    length: u64,
    entry_type: u32,
    reserved: u32,
}

impl MemoryMapEntry {
    pub fn entry_type(&self) -> u32 {
        self.entry_type
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn base_address(&self) -> u64 {
        self.base_address
    }

    pub fn end_address(&self) -> u64 {
        self.base_address + self.length - 1
    }
}

impl fmt::Display for MemoryMapEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "base_address: {},
                length: {},
                entry_type: {},
                reserved: {},",
               self.base_address,
               self.length,
               self.entry_type,
               self.reserved)
    }
}

#[repr(u32)]
pub enum MemoryMapEntryType {
    Reserved = 0,
    Available = 1,
    ACPIInfo = 3,
    ReservedOnHibernation = 4,
}

#[derive(Clone)]
pub struct AvailableMemorySectionsIterator {
    entry_address: usize,
    tag_end_address: usize,
    entry_size: usize,
}

impl AvailableMemorySectionsIterator {
    pub fn new(entry_address: usize, tag_end_address: usize, entry_size: usize) -> AvailableMemorySectionsIterator {
        AvailableMemorySectionsIterator {
            entry_address: entry_address,
            tag_end_address: tag_end_address,
            entry_size: entry_size,
        }
    }
}

impl iter::Iterator for AvailableMemorySectionsIterator {
    type Item = &'static MemoryMapEntry;

    fn next(&mut self) -> Option<&'static MemoryMapEntry> {
        if self.entry_address >= self.tag_end_address {
            None
        } else {
            let result = unsafe { &(*(self.entry_address as *const MemoryMapEntry)) };
            self.entry_address += self.entry_size;
            // skip unused
            // todo possibly replace with loop if this will compile to recursion
            if result.entry_type() != MemoryMapEntryType::Available as u32 {
                self.next()
            } else {
                Some(result)
            }
        }
    }
}