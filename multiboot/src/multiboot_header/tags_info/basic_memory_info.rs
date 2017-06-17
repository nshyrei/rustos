use stdx::conversion::FromAddressToStaticRef;
use core::ptr::read;
use multiboot_header::multiboot_header_tag::MultibootHeaderTag;

pub struct BasicMemoryInfo {
    tag_type: u32,
    tag_size: u32,
    pub memory_lower: u32,
    pub memory_upper: u32,
}

impl FromAddressToStaticRef for BasicMemoryInfo {
    unsafe fn from_unsafe(address: usize) -> &'static BasicMemoryInfo {
        &(*(address as *const BasicMemoryInfo))
    }
}

impl MultibootHeaderTag for BasicMemoryInfo {
    fn numeric_type() -> u32 {
        4
    }
}