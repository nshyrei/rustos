use stdx::conversion::FromUnsafe;
use multiboot_header::tags_info::tag_entry_iterator::TagEntryIterator;
use core::ptr::read;

pub struct MemoryMap {
    pub version: u32,
    pub entries: TagEntryIterator<MemoryMapEntry>,
}

impl FromUnsafe<usize> for MemoryMap {
    unsafe fn from_unsafe(address: usize) -> MemoryMap {
        let (_, tag_size, entry_size, version) = read(address as *const (u32, u32, u32, u32));
        let entry_address = address + 16; //4 u32 fields offset
        let tag_end_address = address + tag_size as usize;

        MemoryMap {
            version: version,
            entries: TagEntryIterator::new(entry_address, tag_end_address, entry_size as usize),
        }
    }
}

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
