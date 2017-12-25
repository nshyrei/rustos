use stdx::ptr;
use core::mem;
use core::ptr::write_unaligned;

pub trait MemoryAllocator {
    fn allocate(&mut self, size : usize) -> Option<usize>;

    fn allocate_from<T>(&mut self) -> Option<usize> {
        self.allocate(mem::size_of::<T>())        
    }

    fn free(&mut self, pointer : usize);

    fn start_address(&self) -> usize;

    fn end_address(&self) -> usize;
}