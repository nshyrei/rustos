use allocator::bump::BumpAllocator;
use stdx_memory::heap;
use stdx_memory::collections::linked_list::LinkedList;
use stdx_memory::MemoryAllocator;
use core::mem;

pub struct FreeListAllocator {
    bump_allocator        : BumpAllocator,
    block_size            : usize,
    free_blocks           : heap::SharedBox<LinkedList<usize>>,
    free_blocks_allocator : BumpAllocator
}

impl FreeListAllocator {
    pub fn from_address(address: usize, size : usize, block_size : usize) -> FreeListAllocator {
        let bump_allocator = BumpAllocator::from_address(address, size);
        let block_count = (address + size - 1) / block_size;
        let free_blocks_list_size = block_count * mem::size_of::<LinkedList<usize>>();

        let mut free_blocks_allocator = BumpAllocator::from_address(bump_allocator.end_address() + 1, free_blocks_list_size);
        let free_blocks = heap::SharedBox::new(LinkedList::Nil, &mut free_blocks_allocator);

        FreeListAllocator {
            bump_allocator        : bump_allocator,
            block_size            : block_size,
            free_blocks           : free_blocks,
            free_blocks_allocator : free_blocks_allocator
        }
    }

    
}

impl MemoryAllocator for FreeListAllocator {
    fn allocate(&mut self, size : usize) -> Option<usize> {
        if let Some((value, previous)) = self.free_blocks.take(&mut self.free_blocks_allocator) {
            self.free_blocks = previous;            
            Some(value)
        }
        else {
            self.bump_allocator.allocate(size)
        }        
    }

    fn free(&mut self, pointer : usize) {
        self.free_blocks = self.free_blocks.add(pointer, &mut self.free_blocks_allocator);
    }

    fn start_address(&self) -> usize {
        1
    }

    fn end_address(&self) -> usize {
        1
    }
}