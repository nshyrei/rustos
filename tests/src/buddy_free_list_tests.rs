use memory::frame::Frame;
use memory::frame::FRAME_SIZE;
use stdx::iterator::IteratorExt;
use stdx::Sequence;
use stdx_memory::MemoryAllocator;
use stdx_memory::collections::double_linked_list::{DoubleLinkedList, DoubleLinkedListIterator, BuddyMap};
use memory::allocator::bump::BumpAllocator;
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
pub fn free_list_should_properly_set_in_use() {
    let (mut buddy_free_list, mut allocator) = init_buddy_map!(2);
    
    buddy_free_list.set_in_use(0, &mut allocator);
    buddy_free_list.set_in_use(1, &mut allocator);

    assert!(buddy_free_list.is_in_use(0), "Failed to set in use for block with start address {}", 0);
    assert!(buddy_free_list.is_in_use(1), "Failed to set in use for block with start address {}", 2);
}

#[test]
pub fn free_list_should_properly_set_free() {


        unsafe {            
            let li : [u8;128] = [0;128];
            let ar : [u8;10000] = [0;10000];
            let cell_size        = BuddyMap::cell_size();
            let array_size       = BuddyMap::mem_size_for_array(2);
            let linked_list_size = BuddyMap::mem_size_for_linked_list(2);
            
            let array_addr = ar.as_ptr() as usize;//heap::allocate_zeroed(array_size, 2);
            let list_addr  = li.as_ptr() as usize;

            let mut array_allocator = BumpAllocator::from_address(array_addr as usize, array_size);
            let mut allocator       = FreeListAllocator::from_address(list_addr as usize, linked_list_size, cell_size);
            let mut buddy_free_list = BuddyMap::new(2, &mut array_allocator, &mut allocator);

         
    buddy_free_list.set_in_use(0, &mut allocator);
    buddy_free_list.set_in_use(1, &mut allocator);

    assert!(buddy_free_list.is_in_use(0), "Failed to set in use for block with start address {}", 0);
    assert!(buddy_free_list.is_in_use(1), "Failed to set in use for block with start address {}", 2);

    buddy_free_list.set_free(0, &mut allocator);
    buddy_free_list.set_free(1, &mut allocator);    

    assert!(buddy_free_list.is_free(0), "Failed to free block with start address {}", 0);
    assert!(buddy_free_list.is_free(1), "Failed to free block with start address {}", 2);
        }
}

#[test]
pub fn set_free_should_properly_remove_elem_in_the_middle_of_the_list() {    
    let (mut buddy_free_list, mut allocator) = init_buddy_map!(3);
        
    buddy_free_list.set_free(0, &mut allocator);
    buddy_free_list.set_free(1, &mut allocator);
    buddy_free_list.set_free(2, &mut allocator);

    buddy_free_list.set_in_use(1, &mut allocator);

    let fst_free = buddy_free_list.first_free_block(&mut allocator);
    let snd_free = buddy_free_list.first_free_block(&mut allocator);
    let thrd_free = buddy_free_list.first_free_block(&mut allocator);

    assert!(fst_free.is_some(), "Failed to return first free block for list 0-4");
    assert!(fst_free.unwrap() == 0, "Returned invalid first free block for list 0-4. Returned {}, but should be {}",
        fst_free.unwrap(),
        0);

    assert!(snd_free.is_some(), "Failed to return first free block for list 4");
    assert!(snd_free.unwrap() == 2, "Returned invalid first free block for list 4. Returned {}, but should be {}",
        fst_free.unwrap(),
        2);

    assert!(thrd_free.is_none(), "Returned value from unknown source for empty list of free blocks. Returned {}",
        thrd_free.unwrap());
}

#[test]
pub fn set_free_should_properly_remove_elem_at_the_start_of_the_list() {    
    let (mut buddy_free_list, mut allocator) = init_buddy_map!(3);
        
    buddy_free_list.set_free(0, &mut allocator);
    buddy_free_list.set_free(1, &mut allocator);
    buddy_free_list.set_free(2, &mut allocator);

    buddy_free_list.set_in_use(0, &mut allocator);

    let fst_free = buddy_free_list.first_free_block(&mut allocator);
    let snd_free = buddy_free_list.first_free_block(&mut allocator);    
    let thrd_free = buddy_free_list.first_free_block(&mut allocator);

    assert!(fst_free.is_some(), "Failed to return first free block for list 2-4");
    assert!(fst_free.unwrap() == 1, "Returned invalid first free block for list 2-4. Returned {}, but should be {}",
        fst_free.unwrap(),
        1);

    assert!(snd_free.is_some(), "Failed to return first free block for list 4");
    assert!(snd_free.unwrap() == 2, "Returned invalid first free block for list 4. Returned {}, but should be {}",
        fst_free.unwrap(),
        2);

    assert!(thrd_free.is_none(), "Returned value from unknown source for empty list of free blocks. Returned {}",
        thrd_free.unwrap());
}

//#[test]
pub fn set_free_should_properly_remove_elem_at_the_end_of_the_list() {        
    let (mut buddy_free_list, mut allocator) = init_buddy_map!(3);
        
    buddy_free_list.set_free(0, &mut allocator);
    buddy_free_list.set_free(1, &mut allocator);
    buddy_free_list.set_free(2, &mut allocator);

    buddy_free_list.set_in_use(2, &mut allocator);

    let fst_free = buddy_free_list.first_free_block(&mut allocator);
    let snd_free = buddy_free_list.first_free_block(&mut allocator);    
    let thrd_free = buddy_free_list.first_free_block(&mut allocator);

    assert!(fst_free.is_some(), "Failed to return first free block for list 0-2");
    assert!(fst_free.unwrap() == 0, "Returned invalid first free block for list 0-2. Returned {}, but should be {}",
        fst_free.unwrap(),
        0);

    assert!(snd_free.is_some(), "Failed to return first free block for list 2");
    assert!(snd_free.unwrap() == 1, "Returned invalid first free block for list 2. Returned {}, but should be {}",
        fst_free.unwrap(),
        1);

    assert!(thrd_free.is_none(), "Returned value from unknown source for empty list of free blocks. Returned {}",
        thrd_free.unwrap());
}