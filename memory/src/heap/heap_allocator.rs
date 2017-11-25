use kernel::bump_allocator::BumpAllocator;



pub struct HeapAllocator {
    heap_start : usize,
    heap_size : usize    
}


impl HeapAllocator {
    pub fn new(frame_bump_allocator : &BumpAllocator) -> HeapAllocator {
        const HEAP_SIZE : usize = 52428800; //50 mb
        let bump_allocator_end_address = frame_bump_allocator.end_address();
        
        HeapAllocator {
            heap_start : bump_allocator_end_address + 1,
            heap_size : HEAP_SIZE
        }
    }
}