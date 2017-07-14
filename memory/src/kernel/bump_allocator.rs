const HEAP_START: usize = 0x40000000;
const HEAP_END : usize = HEAP_START + HEAP_SIZE - 1;
const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub struct BumpAllocator {
    current_pointer: usize,
}

impl BumpAllocator {
    pub const fn new() -> BumpAllocator {
        BumpAllocator { current_pointer: HEAP_START }
    }

    pub fn from_address(address: usize) -> BumpAllocator {
        BumpAllocator { current_pointer: address }
    }

    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        if self.current_pointer + size > HEAP_END {
            None
        }
        else {
            let result = self.current_pointer;
            self.current_pointer += size;

            Some(result)
        }        
    }

    pub fn free(&mut self, size: usize) {        
        self.current_pointer -= size;
    }
}