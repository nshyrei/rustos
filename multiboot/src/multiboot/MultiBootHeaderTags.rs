pub enum MultiBootHeaderTag {
    Unknown,
    End,
    BasicMemoryInfo { memoryLower: u32, memoryUpper: u32 },
    MemoryMap {
        entrySize: u32,
        entryVersion: u32,
        entries: [MemoryMapEntry; 10], //change to dynamic array/collection
    },
}

pub struct BasicMemoryInfo {
    tagType: u32,
    tagSize: u32,
    memoryLower: u32,
    memoryUpper: u32,
}

pub struct MemoryMap {
    tagType: u32,
    tagSize: u32,
    entrySize: u32,
    entryVersion: u32,
}

#[derive(Copy, Clone)]
pub struct MemoryMapEntry {
    baseAddress: u64,
    length: u64,
    entryType: u32,
    reserved: u32,
}

impl MemoryMapEntry {
    pub fn default() -> MemoryMapEntry {
        MemoryMapEntry {
            baseAddress: 0,
            length: 0,
            entryType: 0,
            reserved: 0,
        }
    }
}
