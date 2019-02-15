use allocator::bump::ConstSizeBumpAllocator;
use stdx_memory::heap;
use stdx_memory::collections::linked_list::LinkedList;
use stdx_memory::MemoryAllocator;
use stdx_memory::ConstantSizeMemoryAllocator;
use core::cmp;

#[repr(C)]
pub struct FreeListAllocator {
    bump_allocator        : ConstSizeBumpAllocator,
    block_size            : usize,
    free_blocks           : heap::Box<LinkedList<usize>, ConstSizeBumpAllocator>,
    free_blocks_allocator : ConstSizeBumpAllocator,
    free_blocks_count     : usize
}

impl FreeListAllocator {
    pub fn from_address(address: usize, size : usize, block_size : usize) -> FreeListAllocator {
        let bump_allocator = ConstSizeBumpAllocator::from_address(address, size, block_size);            
        let blocks_count = bump_allocator.total_blocks_count();
        let mut free_blocks_allocator = ConstSizeBumpAllocator::from_address_for_type_multiple::<LinkedList<usize>>(
            bump_allocator.end_address() + 1,
            blocks_count + 1); // +1 for LinkedList::Nil

        let free_blocks = heap::Box::new(LinkedList::Nil, &mut free_blocks_allocator);

        FreeListAllocator {
            bump_allocator,
            block_size,
            free_blocks,
            free_blocks_allocator,
            free_blocks_count :blocks_count
        }
    }

    pub fn fully_free(&self) -> bool {
        self.free_blocks_count == self.bump_allocator.total_blocks_count()
    }

    pub fn fully_occupied(&self) -> bool {
        self.free_blocks_count == 0
    }

    pub fn start_address(&self) -> usize {

        self.bump_allocator.start_address()
    }

    pub fn end_address(&self) -> usize {
        self.free_blocks_allocator.end_address()
    }    

    pub fn increase_size(&mut self, size : usize) {
        let free_list_blocks_increase = size / self.block_size;
        self.bump_allocator.increase_size(size);
        self.free_blocks_allocator.increase_size(free_list_blocks_increase);
    }

    pub fn is_inside_address_space(&self, pointer : usize) -> bool {
        self.bump_allocator.is_inside_address_space(pointer)
    }
}

impl ConstantSizeMemoryAllocator for FreeListAllocator {    
    fn allocate_size(&mut self) -> Option<usize> {
        if let Some((value, previous)) = self.free_blocks.take() {
            self.free_blocks = previous.promote(&mut self.free_blocks_allocator);
            self.free_blocks_count -= 1;

            Some(value)
        }
        else {
            let bump_result = self.bump_allocator.allocate(self.block_size);

            if bump_result.is_some() {
                self.free_blocks_count -= 1;
            }

            bump_result
        }
    }

    fn free_size(&mut self, pointer : usize) {
        self.free_blocks_count += 1;
        self.free_blocks = self.free_blocks.add(pointer, &mut self.free_blocks_allocator);
    }

    fn assigned_memory_size() -> usize {
        unimplemented!()
    }

    fn aux_data_structures_size() -> usize {
        unimplemented!()
    }
}

impl cmp::Ord for FreeListAllocator {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.start_address().cmp(&other.start_address())
    }
}

impl cmp::PartialOrd for FreeListAllocator {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.start_address().partial_cmp(&other.start_address())
    }
}

impl cmp::Eq for FreeListAllocator {

}

impl cmp::PartialEq for FreeListAllocator {
    fn eq(&self, other: &Self) -> bool {
        self.start_address() == other.start_address()
    }
}