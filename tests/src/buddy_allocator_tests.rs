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

#[test]
pub fn should_complete_request_for_all_available_memory() {
    should_complete_request_for_all_available_memory0();
}

fn should_complete_request_for_all_available_memory0() -> (BuddyAllocator, usize, usize, usize, usize) {
    let size = 4096 * 16;
    
    let heap = unsafe { heap::allocate_zeroed(size, 4096) as usize } ;
    let heap_end_address = heap + size - 1;
    let mut allocator = BuddyAllocator::new(heap, heap_end_address);
    
    let result = allocator.allocate(size);

    check_allocation_result(result, heap, heap, heap_end_address, size);

    (allocator, result.unwrap(), heap, heap_end_address, size)
}

fn check_allocation_result(result : Option<usize>, reference : usize, heap : usize, heap_end_address : usize, size : usize){
    assert!(result.is_some(), "Buddy allocator for heap with address: start {}, end {} failed to complete request for size {} (Returned none instead of value)",
        heap,
        heap_end_address,
        size);
    
    assert!(result.unwrap() == reference, 
        "Buddy allocator for heap with address: start {}, end {} Returned block with invalid starting address. Should be {}, but was {}"
        , heap
        , heap_end_address
        , reference
        , result.unwrap());
}

#[test]
pub fn should_return_same_block_after_free_for_all_available_memory() {
    let (mut allocator, result, heap, heap_end_address, size) = should_complete_request_for_all_available_memory0();
    allocator.free(result);

    let result = allocator.allocate(size);

    check_allocation_result(result, heap, heap, heap_end_address, size);
}

#[test]
pub fn should_return_buddy() {
    let size = 4096 * 16;
    
    let heap = unsafe { heap::allocate_zeroed(size, 4096) as usize } ;
    let heap_end_address = heap + size - 1;
    let mut allocator = BuddyAllocator::new(heap, heap_end_address);
    
    allocator.allocate(size / 2);
    let result = allocator.allocate(size / 2);

    check_allocation_result(result, heap + size / 2, heap, heap_end_address, size / 2);    
}

#[test]
pub fn should_merge_buddy_if_possible() {
    let size = 4096 * 16;
    
    let heap = unsafe { heap::allocate_zeroed(size, 4096) as usize } ;
    let heap_end_address = heap + size - 1;
    let mut allocator = BuddyAllocator::new(heap, heap_end_address);
    
    let left = allocator.allocate(size / 2);
    let right = allocator.allocate(size / 2);

    allocator.free(left.unwrap());
    allocator.free(right.unwrap());

    let result = allocator.allocate(size);

    check_allocation_result(result, heap, heap, heap_end_address, size);    
}

#[test]
pub fn should_return_distinct_addresses() {
    let size = 4096 * 16;
    
    let heap = unsafe { heap::allocate_zeroed(size, 4096) as usize } ;
    let heap_end_address = heap + size - 1;
    let mut allocator = BuddyAllocator::new(heap, heap_end_address);
    let mut vec : Vec<usize> = Vec::new();

    for i in 0..16 {
        vec.push(allocator.allocate(4096).unwrap())
    }

    let mut sorted = vec.clone();
    sorted.sort();
    sorted.dedup();

    let result = sorted.len();

    assert!(result == 16, "Allocator failed to allocate 16 distinct blocks of size 4096, returned : {:?}", vec);
}


#[test]
pub fn should_merge_all_buddies_to_the_upper_level() {
    let size = 4096 * 16;
    
    let heap = unsafe { heap::allocate_zeroed(size, 4096) as usize } ;
    let heap_end_address = heap + size - 1;
    let mut allocator = BuddyAllocator::new(heap, heap_end_address);
    let mut allocated : [usize;16] = [0;16];

    for i in 0..16 {
        allocated[i] = allocator.allocate(4096).unwrap();
    }

    for i in 0..16 {
        allocator.free(allocated[i])
    }    

    let result = allocator.allocate(size);

    check_allocation_result(result, heap, heap, heap_end_address, size);    
}

#[test]
pub fn should_satisfy_buddy_request_when_left_block_is_taken() {
    let size = 4096 * 16;
    
    let heap = unsafe { heap::allocate_zeroed(size, 4096) as usize } ;
    let heap_end_address = heap + size - 1;
    let mut allocator = BuddyAllocator::new(heap, heap_end_address);
    let mut allocated : [usize;16] = [0;16];

    for i in 0..8 {
        allocated[i] = allocator.allocate(4096).unwrap();
    }    

    let result = allocator.allocate(size / 2);

    check_allocation_result(result, heap + 4096 * 8, heap, heap_end_address, size / 2);
}