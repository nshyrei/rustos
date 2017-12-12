pub trait MemoryAllocator {
    fn allocate(&mut self, size : usize) -> Option<usize>;

    fn free(&mut self, pointer : usize);
}