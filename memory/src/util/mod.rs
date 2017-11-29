pub mod free_list;
pub mod bump_allocator;
pub mod frame_bitmap;
pub mod free_list_allocator;

pub const HEAP_START: usize = 0x20000000; //start at 512 mb, move to somewhere constant!!!
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB