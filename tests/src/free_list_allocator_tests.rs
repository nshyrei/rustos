use stdx_memory::MemoryAllocator;
use memory::allocator::free_list::FreeListAllocator;

macro_rules! heap_raw {
    ($x:expr) => {{
        let heap = [0;$x];

        heap.as_ptr() as usize
    }}
}

macro_rules! allocator_should_allocate_if_enough_memory0 {
    () => {{
        let heap = heap_raw!(10);
        let block_size = 2;
        let block_count = 10 / block_size;    
        let mut allocator = FreeListAllocator::from_address(heap, 10, block_size);
        let mut allocations : [usize; 5] = [0; 5];
        
        for i in 0..block_count {
            let result = allocator.allocate(0); // doesn't matter what size, because bump works with fixed sizes

            assert!(result.is_some(), "FreeList allocator failed after allocation number {}. Free memaory {}, block size {}",
                i,
                10 - (i * block_size),
                block_size);

            allocations[i] = result.unwrap();
        }

        (allocator, allocations, 10, block_size, block_count)    
    }}
}

#[test]
pub fn allocator_should_allocate_if_enough_memory() {    
    allocator_should_allocate_if_enough_memory0!();
}

#[test]
pub fn allocator_should_allocate_again_after_all_used_memory_is_freed() {
    let (mut allocator, 
         mut allocations, 
         heap_size, 
         block_size, 
         block_count) = allocator_should_allocate_if_enough_memory0!();

    for i in 0..block_count {
        allocator.free(allocations[i]);
    }
    
    for i in 0..block_count {
        let result = allocator.allocate(2);

        assert!(result.is_some(), "FreeList allocator failed after allocation number {}. Free memory {}, block size {}",
            i,
            heap_size - (i * block_size),
            block_size);        
    }
}

#[test]
pub fn allocator_should_not_allocate_if_there_is_no_memory() {
    let heap = heap_raw!(0);
    let mut allocator = FreeListAllocator::from_address(heap, 0, 10);
    
    let result = allocator.allocate(10);

    assert!(result.is_none(), "FreeList allocator allocated memory from unknown source. Test buffer has size {}",
        0);
}

#[test]
pub fn allocator_should_not_allocate_if_there_is_no_more_place_to_allocate() {
    let heap = heap_raw!(5);
    let mut allocator = FreeListAllocator::from_address(heap, 5, 10);
    
    let result = allocator.allocate(10);

    assert!(result.is_none(), "FreeList allocator allocated memory from unknown source. Test buffer has size = {}, when block size is {}",
        0,
        10);
}