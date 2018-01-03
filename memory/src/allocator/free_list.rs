use allocator::bump::BumpAllocator;

pub struct FreeListAllocator {
    bump_allocator : BumpAllocator,

    allocation_size : usize
}

impl FreeListAllocator {
    pub fn from_address(address: usize, size : usize, allocation_size : usize) -> FreeListAllocator {
        FreeListAllocator {
            bump_allocator : BumpAllocator::from_address(address, size),
            allocation_size : allocation_size
        }
    }

    pub fn allocate(&mut self, size : usize) -> Option<usize> {
        None
    }

    pub fn free(&mut self, pointer : usize) {

    }
}