use stdx_memory::MemoryAllocator;
use stdx_memory::ConstantSizeMemoryAllocator;
use stdx_memory::collections::array::Array;
use stdx_memory::collections::double_linked_list::{DoubleLinkedList, DoubleLinkedListCell};
use stdx_memory::heap;
use allocator::bump;
use frame::{Frame, FRAME_SIZE};
use stdx::iterator::IteratorExt;
use allocator::free_list;
use stdx::math;
use core::iter;
use core::mem;

pub struct BuddyAllocator {
    allocation_sizes     : Array<usize>,    
    buddy_free_lists     : Array<BuddyFreeList>,    
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
        let end_address        = Frame::address_align_down(end_address1);
        let total_memory       = end_address - start_address + 1;
        
        let total_frames_count = Frame::from_address(total_memory).number();        
        let total_buddy_levels = BuddyAllocator::index_from_size(total_memory);
        
        let sizes_array_size      = Array::<usize>::mem_size_for(total_frames_count);
        let (buddy_array_size, buddy_free_lists_size) = BuddyAllocator::buddy_free_list_size(
            total_buddy_levels,
            total_memory);
        
        let array_sizes = sizes_array_size + buddy_array_size;

        let mut array_allocator   = bump::BumpAllocator::from_address(start_address, array_sizes);
        let mut free_list_allocator = free_list::FreeListAllocator::from_address(
            array_allocator.end_address() + 1, 
            buddy_free_lists_size,
            BuddyFreeList::cell_size());

        let allocation_sizes      = Array::<usize>::new(total_buddy_levels, &mut array_allocator);        
        let mut buddy_free_lists_array  = Array::<BuddyFreeList>::new(total_buddy_levels, &mut array_allocator);        

        BuddyAllocator::create_buddy_free_lists(
            &mut buddy_free_lists_array, 
            &mut array_allocator, 
            &mut free_list_allocator, 
            total_memory, 
            total_buddy_levels);

        // set initial block that covers all memory as free
        buddy_free_lists_array[total_buddy_levels - 1].set_free(0, &mut free_list_allocator);
                
        BuddyAllocator {
            allocation_sizes            : allocation_sizes,            
            buddy_free_lists            : buddy_free_lists_array,            
            total_memory                : total_memory,            
            array_allocator             : array_allocator,
            free_list_allocator         : free_list_allocator,
            start_address               : start_address
        }
    }

    fn create_buddy_free_lists(buddy_free_lists : &mut Array<BuddyFreeList>, 
        array_allocator : &mut bump::BumpAllocator,
        free_list_allocator : &mut free_list::FreeListAllocator,
        total_memory : usize, 
        total_buddy_levels : usize)
    {
        let it = BlockCountIterator::new(total_memory, total_buddy_levels, FRAME_SIZE).index_items();

        for (block_count, i) in it  {            
            let buddy_free_list = BuddyFreeList::new(block_count, FRAME_SIZE, array_allocator, free_list_allocator);            
            buddy_free_lists.update(i, buddy_free_list);      
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
            else if self.buddy_free_lists[i].has_free_block() {
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

            let left = self.buddy_free_lists[i].first_free_block(&mut self.free_list_allocator).unwrap();

            if allocation_size == BuddyAllocator::block_size_from_index(i as usize) {
                return Some(left)//self.buddy_free_lists[i].first_free_block(&mut self.free_list_allocator)
            }
            else {
                // split buddy
                let lower_level_size = BuddyAllocator::block_size_from_index((i - 1) as usize);
                let right = left + lower_level_size;

                self.buddy_free_lists[i - 1].set_free(right, &mut self.free_list_allocator);

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

            let buddy_free_list = &mut self.buddy_free_lists[buddy_list_index];
            let buddy_index     = buddy_free_list.buddy_index(pointer - self.start_address);

            if !buddy_free_list.is_free(buddy_index) {
                buddy_free_list.set_free(pointer, &mut self.free_list_allocator);
                break;    
            }            
            else {
                buddy_free_list.set_buddy_in_use(pointer, &mut self.free_list_allocator);
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
        let frame_number     = Frame::number_for_address(pointer - self.start_address);
        let buddy_list_index = self.allocation_sizes[frame_number];        

        self.merge_up(pointer, buddy_list_index);
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

pub struct BuddyFreeList {
    frame_to_free_buddy : Array<heap::SharedBox<DoubleLinkedListCell<usize>>>,
    free_blocks         : DoubleLinkedList<usize>,
    block_size          : usize
}

impl BuddyFreeList {
    pub fn new<A, B>(block_count : usize, block_size : usize, memory_allocator : &mut A, list_allocator : &mut B) -> Self 
    where A : MemoryAllocator, B : ConstantSizeMemoryAllocator {
        let mut array = Array::new(block_count, memory_allocator);

        // set list as fully occupied
        for i in 0 .. array.length() {
            array.update(i, heap::SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator));
        }

        BuddyFreeList {
            frame_to_free_buddy : array,
            free_blocks         : DoubleLinkedList::new(list_allocator),
            block_size          : block_size
        }
    }

    fn mem_size_for_array(block_count : usize) -> usize {
        Array::<heap::SharedBox<DoubleLinkedListCell<usize>>>::mem_size_for(block_count)
    }

    fn mem_size_for_linked_list(block_count : usize) -> usize {
        DoubleLinkedList::<usize>::mem_size_for::<usize>(block_count)
    }

    /*
    pub fn mem_size_for(block_count : usize) -> usize {
        let array_size       = Array::<heap::SharedBox<DoubleLinkedListCell<usize>>>::mem_size_for(block_count);
        let linked_list_size = DoubleLinkedList::<usize>::mem_size_for(block_count);

        array_size + linked_list_size
    }
    */   

    pub fn cell_size() -> usize {
        DoubleLinkedList::<usize>::cell_size::<usize>()
    }

    /// Determines if block is free to use
    /// # Arguments
    /// * `block_start_address` - start address of memory block
    pub fn is_free(&self, block_start_address : usize) -> bool {
        !self.is_in_use(block_start_address)
    }

    /// Determines if block is occupied
    /// # Arguments
    /// * `block_start_address` - start address of memory blockfree_list_should_properly_set_free()
    pub fn is_in_use(&self, block_start_address : usize) -> bool {
        // todo block_start_address or frame number will be out of range
        let index = self.address_to_array_index(block_start_address);
        self.is_in_use_with_idx(index)
    }

    fn is_free_with_idx(&self, index : usize) -> bool {
        !self.is_in_use_with_idx(index)
    }

    fn is_in_use_with_idx(&self, index : usize) -> bool {
        self.frame_to_free_buddy.elem_ref(index).is_nil()
    }

    fn buddy_index(&self, block_start_address : usize) -> usize {
        let i = block_start_address / self.block_size;

        if math::is_even(i) {
            i + 1
        }
        else {
            i - 1
        }
    }

    /// Sets the block as occupied
    /// # Arguments    
    /// * `block_start_address` - start address of memory block
    /// * `memory_allocator` - memory allocator
    pub fn set_in_use<A>(&mut self, block_start_address : usize, memory_allocator : &mut A)
    where A : MemoryAllocator {
        let index = self.address_to_array_index(block_start_address);

        if self.is_free_with_idx(index) {
            let cell = self.frame_to_free_buddy.value(index);
            self.remove_free_block(cell, memory_allocator);
            self.frame_to_free_buddy.update(index, heap::SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator));        
        }
    }

    pub fn set_in_use_idx<A>(&mut self, index : usize, memory_allocator : &mut A)
    where A : MemoryAllocator {
        if self.is_free_with_idx(index) {
            let cell = self.frame_to_free_buddy.value(index);
            self.remove_free_block(cell, memory_allocator);
            self.frame_to_free_buddy.update(index, heap::SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator));        
        }
    }

    pub fn set_buddy_in_use<A>(&mut self, block_start_address : usize, memory_allocator : &mut A)
    where A : MemoryAllocator {
        let buddy_index = self.buddy_index(block_start_address);
        self.set_in_use_idx(buddy_index, memory_allocator);
    }

    /// Sets the block as free to use
    /// # Arguments    
    /// * `block_start_address` - start address of memory block
    /// * `memory_allocator` - memory allocator
    pub fn set_free<A>(&mut self, block_start_address : usize, memory_allocator : &mut A) 
    where A : MemoryAllocator {
        let index = self.address_to_array_index(block_start_address);

        if self.is_in_use_with_idx(index) {
            let cell = self.free_blocks.add_to_tail(block_start_address, memory_allocator);            
            self.frame_to_free_buddy.update(index, cell);        
        }
    }

    /// Returns first unused memory block if any.
    /// # Arguments        
    /// * `memory_allocator` - memory allocator
    pub fn first_free_block<A>(&mut self, memory_allocator : &mut A) -> Option<usize> 
    where A : MemoryAllocator{
        let result = self.free_blocks.take_head(memory_allocator);

        if let Some(block_start_address) = result {
            let index = self.address_to_array_index(block_start_address);
            self.frame_to_free_buddy.update(index, heap::SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator));
        }

        result
    }

    pub fn has_free_block(&self) -> bool {
        self.free_blocks.is_cell()
    }

    fn address_to_array_index(&self, address : usize) -> usize {
        address / self.block_size
    }

    fn remove_free_block<A>(&mut self, cell : heap::SharedBox<DoubleLinkedListCell<usize>>, memory_allocator : &mut A)
    where A : MemoryAllocator {
        if self.free_blocks.head_equals_tail() && cell.is_start() {
            self.free_blocks.remove_head(memory_allocator);            
        }
        else if cell.is_start() {
            self.free_blocks.remove_head(memory_allocator);            
        }
        else if cell.is_end() {
            self.free_blocks.remove_tail(memory_allocator);            
        }
        else {
            cell.pointer_mut().remove(memory_allocator);
        }
    }
}