pub struct BumpAllocator {
    current_pointer: usize,
    start_address : usize,
    end_address : usize
}

impl BumpAllocator {
    
    // should be used only for test
    pub fn from_address(address: usize, size : usize) -> BumpAllocator {
        BumpAllocator { current_pointer: address, start_address : address, end_address : address + size - 1 }
    }    

    pub fn current_pointer(&self) -> usize {
        self.current_pointer
    }

    pub fn start_address(&self) -> usize {
        self.start_address
    }

    pub fn end_address(&self) -> usize {
        self.end_address
    }

    pub fn allocate(&mut self, size: usize) -> Option<usize> {        
        if self.current_pointer + size > self.end_address {
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