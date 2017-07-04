use multiboot_header::multiboot_header_tag::MultibootHeaderTag;

#[repr(C)]
#[derive(Debug)]
pub struct BasicMemoryInfo {
    tag_type: u32,
    tag_size: u32,
    pub memory_lower: u32,
    pub memory_upper: u32,
}

impl MultibootHeaderTag for BasicMemoryInfo {
    fn numeric_type() -> u32 {
        4
    }
}