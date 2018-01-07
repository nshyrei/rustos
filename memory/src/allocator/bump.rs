use stdx_memory::MemoryAllocator;

#[derive(Clone)]
pub struct BumpAllocator {
    current_pointer     : usize,
    start_address       : usize,
    pointer_end_address : usize,
    allocation_size     : usize
}

impl BumpAllocator {
        
    pub fn from_address_for_type<T>(address: usize, size : usize) -> Self {
        use core::mem::size_of;
        BumpAllocator::from_address(address, size, size_of::<T>())
    }

    pub fn from_address(address: usize, size : usize, allocation_size : usize) -> Self {
        BumpAllocator { 
            current_pointer     : address, 
            start_address       : address, 
            pointer_end_address : address + size,
            allocation_size     : allocation_size
        }
    }

    pub fn current_pointer(&self) -> usize {
        self.current_pointer
    }
}

impl MemoryAllocator for BumpAllocator {
    fn allocate(&mut self, size: usize) -> Option<usize> {        
        if self.current_pointer + self.allocation_size > self.pointer_end_address {
            None
        }
        else {
            let result = self.current_pointer;
            self.current_pointer += size;

            Some(result)
        }        
    }

    fn free(&mut self, size: usize) {
        self.current_pointer -= self.allocation_size;
    }

    fn start_address(&self) -> usize {
        self.start_address
    }

    fn end_address(&self) -> usize {
        self.pointer_end_address - 1
    }
}