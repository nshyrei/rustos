use multiboot_header::MultibootHeaderTag;
use core::fmt;

#[repr(C)]
pub struct BasicMemoryInfo {
    tag_type: u32,
    tag_size: u32,
    memory_lower: u32,
    memory_upper: u32,
}

impl MultibootHeaderTag for BasicMemoryInfo {
    fn numeric_type() -> u32 {
        4
    }
}

impl fmt::Display for BasicMemoryInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "memory_lower: {},
        memory_upper: {}",
               self.memory_lower,
               self.memory_upper)
    }
}