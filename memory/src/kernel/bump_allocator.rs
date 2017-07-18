const HEAP_START: usize = 0x40000000;
const HEAP_END : usize = HEAP_START + HEAP_SIZE - 1;
const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub struct BumpAllocator {
    current_pointer: usize,
    is_test : bool    
}

impl BumpAllocator {

    pub const fn new() -> BumpAllocator {
        BumpAllocator { current_pointer: HEAP_START, is_test : false }
    }

    // should be used only for test
    pub const fn from_address(address: usize) -> BumpAllocator {
        BumpAllocator { current_pointer: address, is_test : true }
    }    

    pub fn current_pointer(&self) -> usize {
        self.current_pointer
    }

    pub fn allocate(&mut self, size: usize) -> Option<usize> {        
        if !self.is_test && self.current_pointer + size > HEAP_END {
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