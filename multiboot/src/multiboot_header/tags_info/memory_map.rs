use stdx::conversion::FromAddressToStaticRef;
use multiboot_header::tags_info::tag_entry_iterator::TagEntryIterator;
use core::ptr::read;
use multiboot_header::multiboot_header_tag::MultibootHeaderTag;

#[repr(C)] // repr(C) is crucial to make read(address as *const MemoryMap) work properly
// default struct pack couldn't be read like this
pub struct MemoryMap {
    tag_type: u32,
    tag_size: u32,
    entry_size: u32,
    pub version: u32,
    first_entry: MemoryMapEntry,
}

impl MemoryMap {
    pub fn entries(&self) -> TagEntryIterator<MemoryMapEntry> {
        let entry_address = (&self.first_entry) as *const _ as usize;
        let tag_end_address = (self as *const _ as usize) + self.tag_size as usize;
        TagEntryIterator::new(entry_address, tag_end_address, self.entry_size as usize)
    }
}

impl FromAddressToStaticRef for MemoryMap {
    unsafe fn from_unsafe(address: usize) -> &'static MemoryMap {
        &(*(address as *const MemoryMap))
    }
}

impl MultibootHeaderTag for MemoryMap {
    fn numeric_type() -> u32 {
        6
    }
}

#[repr(C)]
pub struct MemoryMapEntry {
    pub base_address: u64,
    pub length: u64,
    pub entry_type: u32,
    reserved: u32,
}

impl FromAddressToStaticRef for MemoryMapEntry {
    unsafe fn from_unsafe(address: usize) -> &'static MemoryMapEntry {
        &(*(address as *const MemoryMapEntry))
    }
}