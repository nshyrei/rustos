use memory::frame::Frame;
use memory::frame::FRAME_SIZE;
use stdx::iterator::IteratorExt;
use stdx::Sequence;
use stdx_memory::MemoryAllocator;
use stdx_memory::collections::double_linked_list::{DoubleLinkedList, DoubleLinkedListIterator, BuddyMap};
use memory::allocator::bump::BumpAllocator;
use memory::allocator::buddy::BuddyAllocator;
use memory::allocator::free_list::FreeListAllocator;
use std::mem;
use alloc::heap;

macro_rules! init_buddy_map {
    ($l:expr) => {{

        unsafe {            

            let cell_size        = BuddyMap::cell_size();
            let array_size       = BuddyMap::mem_size_for_array($l);
            let linked_list_size = BuddyMap::mem_size_for_linked_list($l);
            
            let array_addr = heap::allocate_zeroed(array_size, 2);
            let list_addr  = heap::allocate_zeroed(linked_list_size, 2);

            let mut array_allocator = BumpAllocator::from_address(array_addr as usize, array_size);
            let mut allocator       = FreeListAllocator::from_address(list_addr as usize, linked_list_size, cell_size);
            let mut buddy_free_list = BuddyMap::new($l, &mut array_allocator, &mut allocator);

            (buddy_free_list, allocator)
        }
    }}
}

#[test]
#[should_panic]
pub fn should_not_create_allocator_if_there_is_no_memory() {    
    let mut allocator = BuddyAllocator::new(0, 0);    
}

#[test]
#[should_panic]
pub fn should_not_create_allocator_if_there_is_no_memory2() {
    let heap : [u8;40960] = [0;40960];
    let heap_addr = heap.as_ptr() as usize;

    let mut allocator = BuddyAllocator::new(heap_addr, 0);    
}

#[test]
pub fn should_return_none_if_requested_more_then_available_memory() {
    let heap = unsafe { heap::allocate_zeroed(4096, 4096) as usize } ;

    let mut allocator = BuddyAllocator::new(heap, heap + 4095);
    
    let result = allocator.allocate(100000000);
    
    assert!(result.is_none(), 
        "Buddy allocator for heap with address: start {}, end {} allocated memory from unknown place for size {}. Allocator returned {}"
        , heap
        , heap + 5
        , 100000000
        , result.unwrap());    
}