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
    allocation_sizes    : Array<usize>,    
    buddy_free_lists    : Array<BuddyFreeList>,    
    array_allocator     : bump::BumpAllocator,
    free_list_allocator : free_list::FreeListAllocator,
    memory_size         : usize,
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
        let memory_size        = end_address - start_address + 1;
        
        let total_frames_count = Frame::from_address(memory_size).number();        
        let total_buddy_levels = BuddyAllocator::buddy_index(memory_size);
        
        let sizes_array_size      = Array::<usize>::mem_size_for(total_frames_count);
        let (buddy_array_size, buddy_free_lists_size) = BuddyAllocator::buddy_bitmaps_size(total_buddy_levels, total_frames_count);
        
        let array_sizes = sizes_array_size + buddy_free_lists_size;

        let mut array_allocator   = bump::BumpAllocator::from_address(start_address, array_sizes);
        let mut free_list_allocator = free_list::FreeListAllocator::from_address(
            array_allocator.end_address() + 1, 
            buddy_free_lists_size,
            BuddyFreeList::cell_size());

        let allocation_sizes      = Array::<usize>::new(total_buddy_levels, &mut array_allocator);        
        let mut buddy_free_lists  = Array::<BuddyFreeList>::new(total_buddy_levels, &mut array_allocator);        

        let mut block_size = FRAME_SIZE;        

        for i in 0 .. total_buddy_levels {
            let block_count = memory_size / block_size;                      
            
            let buddy_free_list = BuddyFreeList::new(block_count, block_size, &mut array_allocator, &mut free_list_allocator);
            
            buddy_free_lists.update(i, buddy_free_list);
            
            block_size *= 2;
        }

        // set initial block that covers all memory as free
        buddy_free_lists.elem_ref_mut(total_buddy_levels - 1).set_free(0, &mut free_list_allocator);
                
        BuddyAllocator {
            allocation_sizes            : allocation_sizes,            
            buddy_free_lists            : buddy_free_lists,            
            memory_size                 : memory_size,            
            array_allocator             : array_allocator,
            free_list_allocator         : free_list_allocator
        }
    }

    fn buddy_bitmaps_size(buddy_levels_count : usize, total_memory : usize) -> (usize, usize) {
        let mut array_size = 0;
        let mut free_list_size = 0;
        let mut block_size = FRAME_SIZE;

        for _ in 0 .. buddy_levels_count {
            let block_count = total_memory / block_size;            
            free_list_size += BuddyFreeList::mem_size_for_linked_list(block_count);
            array_size += BuddyFreeList::mem_size_for_array(block_count);
            block_size *= 2;
        }
        
        (array_size, free_list_size)
    }    

    fn buddy_index(block_size : usize) -> usize {
        math::log2(block_size) - 12 // 2 ^ 12 = 4096 = FRAME_SIZE
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
        (2 as usize).pow((12 + buddy_index + 1) as u32)
    }

    fn split(&mut self, allocation_size : usize, buddy_index : isize) -> Option<usize> {
        let mut i = buddy_index;

        loop {            
            
            if i < 0 {
                return None
            }
            else if allocation_size == BuddyAllocator::block_size_from_index(i) {
                return self.buddy_free_lists[i].first_free_block(&mut self.free_list_allocator)
            }
            else {
                let left = self.buddy_free_lists[i].first_free_block(&mut self.free_list_allocator).unwrap();

                // split buddy
                let lower_level_size = BuddyAllocator::block_size_from_index(i - 1);
                let right = left + lower_level_size;

                self.buddy_free_lists[i - 1].set_free(right, &mut self.free_list_allocator);
                self.buddy_free_lists[i - 1].set_free(left, &mut self.free_list_allocator);                            
                
                i -= 1;    
            }

        }
    }    
}

impl MemoryAllocator for BuddyAllocator {

    fn allocate(&mut self, size : usize) -> Option<usize> {

        let allocation_size_rounded = Frame::address_align_up(size);

        if allocation_size_rounded > self.memory_size {
            None
        }
        else {
            // search free list that corresponds to 'closest_block_size'
            // if there are no entries search free list array upwards
            // to find bigger block to split
            let buddy_index = BuddyAllocator::buddy_index(allocation_size_rounded);
            match self.search_free_list_up(buddy_index) {
                Some(buddy_index_with_free_block) => {
                    self.split(allocation_size_rounded, buddy_index)
                },
                None => None
            }
        }
    }

    fn free(&mut self, pointer : usize) {

    }    
}

struct BuddyLevelSizesIterator {
    total_memory : usize,
    pow : usize,    
}

impl BuddyLevelSizesIterator {
    fn new(total_memory : usize) -> Self {
        BuddyLevelSizesIterator {
            total_memory : total_memory,
            pow : FRAME_SIZE
        }
    }
}

impl iter::Iterator for BuddyLevelSizesIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.pow <= self.total_memory {
            let result = self.total_memory / self.pow;
            self.pow *= 2;

            Some(result)
        }
        else {
            None
        }
    }
}

impl IteratorExt for BuddyLevelSizesIterator {}

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