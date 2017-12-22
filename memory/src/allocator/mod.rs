pub trait MemoryAllocator {
    fn allocate(&mut self, size : usize) -> Option<usize>;

    fn free(&mut self, pointer : usize);

    fn start_address(&self) -> usize;

    fn end_address(&self) -> usize;
}