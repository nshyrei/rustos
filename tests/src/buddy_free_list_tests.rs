use memory::frame::Frame;
use memory::frame::FRAME_SIZE;
use stdx::iterator::IteratorExt;
use stdx_memory::MemoryAllocator;
use stdx_memory::collections::double_linked_list::{DoubleLinkedListCell, DoubleLinkedList, DoubleLinkedListIterator};
use memory::allocator::bump::BumpAllocator;
use memory::allocator::free_list::FreeListAllocator;
use memory::allocator::buddy::BuddyFreeList;
use std::mem;

macro_rules! heap_raw {
    ($x:expr) => {{
        let heap = [0;$x];    

        (heap.as_ptr() as usize, $x)
    }}    
}

#[test]
pub fn free_list_should_properly_set_in_use() {
    let (heap_start, heap_size) = heap_raw!(200); //todo rework to size computation
    let size = mem::size_of::<DoubleLinkedListCell<u8>>();
    let mut allocator = FreeListAllocator::from_address(heap_start, 200, mem::size_of::<DoubleLinkedListCell<u8>>());
    let mut buddy_free_list = BuddyFreeList::new(2, 2, &mut allocator);

    buddy_free_list.set_in_use(0, &mut allocator);
    buddy_free_list.set_in_use(2, &mut allocator);

    assert!(buddy_free_list.is_in_use(0), "Failed to set in use for block with start address {}", 0);
    assert!(buddy_free_list.is_in_use(2), "Failed to set in use for block with start address {}", 2);
}

#[test]
pub fn free_list_should_properly_set_free() {    
    let heap = [0;400];    
    
    let (heap_start, heap_size) = (heap.as_ptr() as usize, 400);
    
    let size = mem::size_of::<DoubleLinkedListCell<u8>>();
    let mut allocator = FreeListAllocator::from_address(heap_start, 400, mem::size_of::<DoubleLinkedListCell<u8>>());
    let mut buddy_free_list = BuddyFreeList::new(2, 2, &mut allocator);

    buddy_free_list.set_in_use(0, &mut allocator);
    buddy_free_list.set_in_use(2, &mut allocator);

    assert!(buddy_free_list.is_in_use(0), "Failed to set in use for block with start address {}", 0);
    assert!(buddy_free_list.is_in_use(2), "Failed to set in use for block with start address {}", 2);

    buddy_free_list.set_free(0, &mut allocator);
    buddy_free_list.set_free(2, &mut allocator);    

    assert!(buddy_free_list.is_free(0), "Failed to free block with start address {}", 0);
    assert!(buddy_free_list.is_free(2), "Failed to free block with start address {}", 2);
}

#[test]
pub fn set_free_should_properly_remove_elem_in_the_middle_of_the_list() {    
    let heap = [0;400];    
    
    let (heap_start, heap_size) = (heap.as_ptr() as usize, 400);
    
    let size = mem::size_of::<DoubleLinkedListCell<u8>>();
    let mut allocator = FreeListAllocator::from_address(heap_start, 400, mem::size_of::<DoubleLinkedListCell<u8>>());
    let mut buddy_free_list = BuddyFreeList::new(3, 2, &mut allocator);
        
    buddy_free_list.set_free(0, &mut allocator);
    buddy_free_list.set_free(2, &mut allocator);
    buddy_free_list.set_free(4, &mut allocator);

    buddy_free_list.set_in_use(2, &mut allocator);

    let fst_free = buddy_free_list.first_free_block(&mut allocator);
    let snd_free = buddy_free_list.first_free_block(&mut allocator);
    let thrd_free = buddy_free_list.first_free_block(&mut allocator);

    assert!(fst_free.is_some(), "Failed to return first free block for list 0-4");
    assert!(fst_free.unwrap() == 0, "Returned invalid first free block for list 0-4. Returned {}, but should be {}",
        fst_free.unwrap(),
        0);

    assert!(snd_free.is_some(), "Failed to return first free block for list 4");
    assert!(snd_free.unwrap() == 4, "Returned invalid first free block for list 4. Returned {}, but should be {}",
        fst_free.unwrap(),
        4);

    assert!(thrd_free.is_none(), "Returned value from unknown source for empty list of free blocks. Returned {}",
        thrd_free.unwrap());
}

#[test]
pub fn set_free_should_properly_remove_elem_at_the_start_of_the_list() {    
    let heap = [0;400];    
    
    let (heap_start, heap_size) = (heap.as_ptr() as usize, 400);
    
    let size = mem::size_of::<DoubleLinkedListCell<u8>>();
    let mut allocator = FreeListAllocator::from_address(heap_start, 400, mem::size_of::<DoubleLinkedListCell<u8>>());
    let mut buddy_free_list = BuddyFreeList::new(3, 2, &mut allocator);
        
    buddy_free_list.set_free(0, &mut allocator);
    buddy_free_list.set_free(2, &mut allocator);
    buddy_free_list.set_free(4, &mut allocator);

    buddy_free_list.set_in_use(0, &mut allocator);

    let fst_free = buddy_free_list.first_free_block(&mut allocator);
    let snd_free = buddy_free_list.first_free_block(&mut allocator);    
    let thrd_free = buddy_free_list.first_free_block(&mut allocator);

    assert!(fst_free.is_some(), "Failed to return first free block for list 2-4");
    assert!(fst_free.unwrap() == 2, "Returned invalid first free block for list 2-4. Returned {}, but should be {}",
        fst_free.unwrap(),
        2);

    assert!(snd_free.is_some(), "Failed to return first free block for list 4");
    assert!(snd_free.unwrap() == 4, "Returned invalid first free block for list 4. Returned {}, but should be {}",
        fst_free.unwrap(),
        4);

    assert!(thrd_free.is_none(), "Returned value from unknown source for empty list of free blocks. Returned {}",
        thrd_free.unwrap());
}

#[test]
pub fn set_free_should_properly_remove_elem_at_the_end_of_the_list() {        
    let heap = [0;400];    
    
    let (heap_start, heap_size) = (heap.as_ptr() as usize, 400);
    
    let size = mem::size_of::<DoubleLinkedListCell<u8>>();
    let mut allocator = FreeListAllocator::from_address(heap_start, 400, mem::size_of::<DoubleLinkedListCell<u8>>());
    let mut buddy_free_list = BuddyFreeList::new(3, 2, &mut allocator);
        
    buddy_free_list.set_free(0, &mut allocator);
    buddy_free_list.set_free(2, &mut allocator);
    buddy_free_list.set_free(4, &mut allocator);

    buddy_free_list.set_in_use(4, &mut allocator);

    let fst_free = buddy_free_list.first_free_block(&mut allocator);
    let snd_free = buddy_free_list.first_free_block(&mut allocator);    
    let thrd_free = buddy_free_list.first_free_block(&mut allocator);

    assert!(fst_free.is_some(), "Failed to return first free block for list 0-2");
    assert!(fst_free.unwrap() == 0, "Returned invalid first free block for list 0-2. Returned {}, but should be {}",
        fst_free.unwrap(),
        0);

    assert!(snd_free.is_some(), "Failed to return first free block for list 2");
    assert!(snd_free.unwrap() == 2, "Returned invalid first free block for list 2. Returned {}, but should be {}",
        fst_free.unwrap(),
        2);

    assert!(thrd_free.is_none(), "Returned value from unknown source for empty list of free blocks. Returned {}",
        thrd_free.unwrap());
}