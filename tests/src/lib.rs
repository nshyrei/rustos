extern crate memory;
extern crate multiboot;
extern crate stdx_memory;
extern crate stdx;

#[cfg(test)]
mod frame_bitmap_tests;
mod frame_allocator_tests;
mod linked_list_tests;
mod double_linked_list_tests;
mod free_list_allocator_tests;
mod buddy_free_list_tests;
mod buddy_allocator_tests;
