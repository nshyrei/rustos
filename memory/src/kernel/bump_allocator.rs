const HEAP_START: usize = 0x40000000;
const HEAP_SIZE: usize = 100 * 1024; // 100 KiB



pub struct BumpAllocator {
    current_pointer: usize,
}


impl BumpAllocator {
    pub fn new() -> BumpAllocator {
        BumpAllocator { current_pointer: HEAP_START }
    }

    pub fn allocate(&mut self, size: usize) -> usize {
        let result = self.current_pointer;
        self.current_pointer += size;

        result
    }
}