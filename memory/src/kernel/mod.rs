pub mod empty_frame_list;
pub mod bump_allocator;
pub mod frame_bitmap;

use kernel::bump_allocator::BumpAllocator;

pub static mut KERNEL_BASIC_HEAP_ALLOCATOR : BumpAllocator = BumpAllocator::new();
