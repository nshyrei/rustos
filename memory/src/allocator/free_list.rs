use allocator::bump::ConstSizeBumpAllocator;
use stdx_memory::heap;
use stdx_memory::collections::linked_list::LinkedList;
use stdx_memory::MemoryAllocator;
use stdx_memory::ConstantSizeMemoryAllocator;

pub struct FreeListAllocator {
    bump_allocator        : ConstSizeBumpAllocator,
    block_size            : usize,
    free_blocks           : heap::SharedBox<LinkedList<usize>>,
    free_blocks_allocator : ConstSizeBumpAllocator
}

impl FreeListAllocator {
    pub fn from_address(address: usize, size : usize, block_size : usize) -> FreeListAllocator {
        let bump_allocator = ConstSizeBumpAllocator::from_address(address, size, block_size);            

        let mut free_blocks_allocator = ConstSizeBumpAllocator::from_address_for_type_multiple::<LinkedList<usize>>(
            bump_allocator.end_address() + 1, 
            bump_allocator.total_blocks_count() + 1); // +1 for LinkedList::Nil        

        let free_blocks = heap::SharedBox::new(LinkedList::Nil, &mut free_blocks_allocator);

        FreeListAllocator {
            bump_allocator        : bump_allocator,
            block_size            : block_size,
            free_blocks           : free_blocks,
            free_blocks_allocator : free_blocks_allocator
        }
    }

    pub fn start_address(&self) -> usize {
        1
    }

    pub fn end_address(&self) -> usize {
        1
    }    
}

impl ConstantSizeMemoryAllocator for FreeListAllocator {    
    fn allocate_size(&mut self) -> Option<usize> {
        if let Some((value, previous)) = self.free_blocks.take(&mut self.free_blocks_allocator) {
            self.free_blocks = previous;            
            Some(value)
        }
        else {
            self.bump_allocator.allocate(self.block_size)
        }
    }

    fn free_size(&mut self, pointer : usize) {
        self.free_blocks = self.free_blocks.add(pointer, &mut self.free_blocks_allocator);
    }

}