use stdx_memory::MemoryAllocator;
use stdx_memory::ConstantSizeMemoryAllocator;
use stdx_memory::collections::array::Array;
use stdx_memory::collections::double_linked_list::{DoubleLinkedList, BuddyFreeList};
use stdx_memory::heap;
use allocator::bump;
use frame::{Frame, FRAME_SIZE};
use stdx::iterator::IteratorExt;
use allocator::free_list;
use stdx::math;
use stdx::Sequence;
use core::iter;
use core::mem;

pub struct BuddyAllocator {
    allocation_sizes     : Array<usize>,    
    buddy_free_lists     : Array<BuddyFreeList0>,    
    array_allocator      : bump::BumpAllocator,
    free_list_allocator  : free_list::FreeListAllocator,
    total_memory         : usize,
    start_address        : usize
}

impl BuddyAllocator {

    fn start_address(&self) -> usize {
        1
    }

    fn end_address(&self) -> usize {
        1
    }

    pub unsafe fn new(start_address1 : usize, end_address1 : usize) -> BuddyAllocator {
        let start_address      = Frame::address_align_up(start_address1);
        let end_address        = Frame::address_align(end_address1);
        let total_memory       = end_address - start_address + 1;
        
        let total_frames_count = Frame::from_address(total_memory).number();        
        let total_buddy_levels = BuddyAllocator::index_from_size(total_memory);
        
        let sizes_array_size      = Array::<usize>::mem_size_for(total_frames_count);
        let (buddy_array_size, buddy_free_lists_size) = BuddyAllocator::buddy_free_list_size(
            total_buddy_levels,
            total_memory);
        
        let array_sizes = sizes_array_size + buddy_array_size;

        let mut array_allocator     = bump::BumpAllocator::from_address(start_address, array_sizes);
        let mut free_list_allocator = free_list::FreeListAllocator::from_address(
            array_allocator.end_address() + 1, 
            buddy_free_lists_size,
            BuddyFreeList::cell_size());

        let allocation_sizes            = Array::<usize>::new(total_buddy_levels, &mut array_allocator);        
        let mut buddy_free_lists_array  = Array::<BuddyFreeList0>::new(total_buddy_levels, &mut array_allocator);        

        BuddyAllocator::create_buddy_free_lists(
            &mut buddy_free_lists_array, 
            &mut array_allocator, 
            &mut free_list_allocator, 
            total_memory, 
            total_buddy_levels);

        // set initial block that covers all memory as free
        buddy_free_lists_array[total_buddy_levels - 1].0.set_free(0, &mut free_list_allocator);
                
        BuddyAllocator {
            allocation_sizes            : allocation_sizes,            
            buddy_free_lists            : buddy_free_lists_array,            
            total_memory                : total_memory,            
            array_allocator             : array_allocator,
            free_list_allocator         : free_list_allocator,
            start_address               : start_address
        }
    }

    fn create_buddy_free_lists(buddy_free_lists : &mut Array<BuddyFreeList0>, 
        array_allocator : &mut bump::BumpAllocator,
        free_list_allocator : &mut free_list::FreeListAllocator,
        total_memory : usize, 
        total_buddy_levels : usize)
    {
        let it = BlockCountIterator::new(total_memory, total_buddy_levels, FRAME_SIZE).index_items();

        for (block_count, i) in it {
            let buddy_free_list = BuddyFreeList::new(block_count, array_allocator, free_list_allocator);            
            buddy_free_lists.update(i, BuddyFreeList0(buddy_free_list));      
        }
    }

    fn buddy_free_list_size(buddy_levels_count : usize, total_memory : usize) -> (usize, usize) {
        let mut array_size = 0;
        let mut free_list_size = 0;        

        for block_count in BlockCountIterator::new(total_memory, buddy_levels_count, FRAME_SIZE) {            
            free_list_size += BuddyFreeList::mem_size_for_linked_list(block_count);
            array_size += BuddyFreeList::mem_size_for_array(block_count);            
        }
        
        (array_size, free_list_size)
    }  

    fn search_free_list_up(&self, index_from : usize) -> Option<usize> {
        let list_length = self.buddy_free_lists.length();        
        let mut i       = index_from;

        loop {
            if i > list_length - 1 {
                return None
            }
            else if self.buddy_free_lists[i].0.has_free_block() {
                return Some(i)
            }
            else {
                i += 1;
            } 
        }   
    }

    fn block_size_from_index(buddy_index : usize) -> usize {
        // 2 ^ 12 = 4096 = FRAME_SIZE
        // (2 as usize).pow((12 + buddy_index) as u32)
        1 << (12 + buddy_index)
    }

    fn index_from_size(block_size : usize) -> usize {
        math::log2(block_size) - 12 // 2 ^ 12 = 4096 = FRAME_SIZE
    }    

    fn split_down(&mut self, allocation_size : usize, start_index : usize) -> Option<usize> {
        let mut i = start_index as isize;        

        loop {
            
            if i < 0 {
                return None
            }

            let current_level_size = BuddyAllocator::block_size_from_index(i as usize);
            let left = self.buddy_free_lists[i].0.first_free_block(&mut self.free_list_allocator).unwrap();

            if allocation_size == current_level_size {
                return Some(left * current_level_size)
            }
            else {
                // split buddy
                let left_start_address  = left * current_level_size;
                let lower_level_size    = BuddyAllocator::block_size_from_index((i - 1) as usize);
                let right_start_address = left_start_address + lower_level_size;
                let right               = right_start_address / lower_level_size;

                self.buddy_free_lists[i - 1].0.set_free(right, &mut self.free_list_allocator);

                // No point in setting 'left' to 'in use'. The reason for that is after split there are only two 
                // operations that could be done with 'left' : splitting again , which will set it to 'in use'
                // and returning it as a result, which also sets it to 'in use'.
                //self.buddy_free_lists[i - 1].set_free(left, &mut self.free_list_allocator);                            
                                
                i -= 1;
            }
        }
    }
    
    fn merge_up(&mut self, pointer : usize, start_index : usize) {        
        let buddy_lists_count    = self.buddy_free_lists.length();
        let mut buddy_list_index = start_index;

        loop {
            if buddy_list_index > buddy_lists_count - 1 {
                break;
            }

            let buddy_free_list            = &mut self.buddy_free_lists[buddy_list_index];
            let buddy_free_list_block_size = BuddyAllocator::block_size_from_index(buddy_list_index);
            let buddy_index                = pointer / buddy_free_list_block_size;

            if !buddy_free_list.0.is_free(buddy_index) {
                buddy_free_list.0.set_free(buddy_index, &mut self.free_list_allocator);
                break;    
            }            
            else {
                buddy_free_list.set_buddy_in_use(buddy_index, &mut self.free_list_allocator);
                buddy_list_index += 1;
            }
        };
    }    
}

impl MemoryAllocator for BuddyAllocator {

    fn allocate(&mut self, size : usize) -> Option<usize> {

        let allocation_size_rounded = Frame::address_align_up(size);

        if allocation_size_rounded > self.total_memory {
            None
        }
        else {
            
            let buddy_index = BuddyAllocator::index_from_size(allocation_size_rounded);
            
            // Search buddy tree for free blocks on current level denoted by 'buddy_index',
            // if nothing found search buddy tree upwards for bigger block that can be splitted.
            // Split bigger block (if any) and propagate split results downwards,
            // until block of required size is created.
            let result = self.search_free_list_up(buddy_index)
                             .and_then(|index_with_free_block| {
                                self.split_down(allocation_size_rounded, index_with_free_block)
                             });
            
            if let Some(block_address) = result {
                // save allocated size by address
                let frame_number = Frame::number_for_address(block_address);
                self.allocation_sizes[frame_number] = buddy_index;

                result
            }
            else {
                None
            }
        }
    }

    fn free(&mut self, pointer : usize) {
        let normalized_pointer = pointer - self.start_address;
        let frame_number       = Frame::number_for_address(normalized_pointer);
        let buddy_list_index   = self.allocation_sizes[frame_number];        

        self.merge_up(normalized_pointer, buddy_list_index);
    }
}

struct BlockCountIterator {
    total_buddy_levels : usize,
    block_size         : usize,
    i                  : usize,
    total_memory       : usize    
}

impl BlockCountIterator {
    fn new(total_memory : usize, total_buddy_levels : usize, starting_block_size : usize) -> Self {
        BlockCountIterator {
            total_buddy_levels : total_buddy_levels,
            block_size         : starting_block_size,
            i                  : 0,
            total_memory       : total_memory
        }
    }
}

impl iter::Iterator for BlockCountIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.i < self.total_buddy_levels {
            let result = self.total_memory / self.block_size;
            self.block_size *= 2;

            Some(result)
        }
        else {
            None
        }
    }
}

impl IteratorExt for BlockCountIterator {}

struct BuddyFreeList0(BuddyFreeList);

impl BuddyFreeList0 {
    fn buddy_index(i : usize) -> usize {        
        if math::is_even(i) {
            i + 1
        }
        else {
            i - 1
        }
    }

    fn set_buddy_in_use<A>(&mut self, index : usize, memory_allocator : &mut A)
    where A : MemoryAllocator {
        let buddy_index = BuddyFreeList0::buddy_index(index);
        self.0.set_in_use(buddy_index, memory_allocator);
    }
}