use allocator::MemoryAllocator;
use util::array::{Array, ArrayIterator};
use util::frame_bitmap::FrameBitMap;
use util::free_list::FreeList;
use util::bump_allocator::BumpAllocator;
use frame::{Frame, FRAME_SIZE};
use util::Box;
use util::SharedBox;
use stdx::iterator::IteratorExt;
use stdx::math;
use core::iter;
use core::mem;

pub struct BuddyAllocator {
    allocation_sizes : Array<usize>,
    buddy_bitmaps    : Array<FrameBitMap>,
    buddy_free_lists : Array<Option<FreeBlocksList>>,
    buddy_free_lists_allocators : Array<BumpAllocator>,
    memory_size     : usize,    
    memory_allocator : BumpAllocator
}

type FreeBlocksList = SharedBox<FreeList<usize>>;

impl BuddyAllocator {
    pub unsafe fn new(start_address1 : usize, end_address1 : usize) -> BuddyAllocator {
        let start_address      = Frame::address_align_up(start_address1);
        let end_address        = Frame::address_align_down(end_address1);
        let memory_size        = end_address - start_address + 1;
        
        let total_frames_count = Frame::from_address(memory_size).number();        
        let total_buddy_levels = BuddyAllocator::buddy_index(memory_size);
        
        let sizes_array_size = mem::size_of::<usize>() * total_frames_count;
        let (buddy_bitmaps_size, buddy_free_lists_size) = BuddyAllocator::buddy_bitmaps_size(total_buddy_levels, total_frames_count);
        
        let aux_data_structures_size = sizes_array_size + buddy_bitmaps_size + buddy_free_lists_size; 

        let mut memory_allocator  = BumpAllocator::from_address(start_address, aux_data_structures_size);
        
        let allocation_sizes      = Array::<usize>::new(total_buddy_levels, &mut memory_allocator);
        let mut buddy_bitmaps     = Array::<FrameBitMap>::new(total_buddy_levels, &mut memory_allocator);
        let mut buddy_free_lists  = Array::<Option<SharedBox<FreeList<usize>>>>::new(total_buddy_levels, &mut memory_allocator);
        let mut buddy_free_lists_allocators = Array::<BumpAllocator>::new(total_buddy_levels, &mut memory_allocator);        

        let mut block_size = FRAME_SIZE;
        let mut previous_free_list_alloc_end_address = memory_allocator.end_address();
        
        for i in 0 .. total_buddy_levels {
            let block_count = memory_size / block_size;
            let free_list_size = block_count * mem::size_of::<FreeList<Frame>>();
            let free_list_allocator = BumpAllocator::from_address(previous_free_list_alloc_end_address + 1, free_list_size);

            previous_free_list_alloc_end_address = free_list_allocator.end_address();

            let bitmap = FrameBitMap::new(block_count, &mut memory_allocator);

            buddy_bitmaps.update(i, bitmap);
            buddy_free_lists_allocators.update(i, free_list_allocator);
            buddy_free_lists.update(i, None);

            block_size *= 2;
        }

        {
            let mut top_level_free_list_allocator = buddy_free_lists_allocators.elem_ref_mut(total_buddy_levels - 1);
            let top_level_free_list = FreeList::new(start_address, &mut top_level_free_list_allocator);
            buddy_free_lists.update(total_buddy_levels - 1, Some(top_level_free_list));
        }
        
        BuddyAllocator {
            allocation_sizes            : allocation_sizes,
            buddy_bitmaps               : buddy_bitmaps,
            buddy_free_lists            : buddy_free_lists,
            buddy_free_lists_allocators : buddy_free_lists_allocators,
            memory_size                 : memory_size,            
            memory_allocator            : memory_allocator
        }
    }

    fn buddy_bitmaps_size(buddy_levels_count : usize, total_memory : usize) -> (usize, usize) {

        let mut bitmaps_size = 0;
        let mut free_list_size = 0;
        let mut block_size = FRAME_SIZE;

        for _ in 0 .. buddy_levels_count {
            let block_count = total_memory / block_size;
            let free_list_cell_size = mem::size_of::<FreeList<Frame>>() + mem::size_of::<Option<Box<FreeList<Frame>>>>() + mem::size_of::<Box<FreeList<Frame>>>();
            bitmaps_size += FrameBitMap::cell_size(block_count) + mem::size_of::<FrameBitMap>();
            free_list_size += block_count * free_list_cell_size;
            block_size *= 2;
        }
        
        (bitmaps_size, free_list_size)
    }    

    fn buddy_index(block_size : usize) -> usize {
        math::log2(block_size) - 12 // 2 ^ 12 = 4096 = FRAME_SIZE
    }

    fn search_free_list_up(&self, index_from : usize) -> Option<(usize, Box<FreeList<usize>>)> {
        let list_length = self.buddy_free_lists.length();        
        let mut result : Option<Box<FreeList<usize>>> = None;
        let mut i = index_from;

        while i < list_length && result.is_none() {
          //  result = self.buddy_free_lists.elem_ref(i);
            i += 1;
        }

        None
        //result.map(|e| (i - 1, e))
    }

    fn block_size_from_index(buddy_index : usize) -> usize {
        (2 as usize).pow((12 + buddy_index + 1) as u32)
    }

    fn split(&mut self, allocation_size : usize, buddy_index : usize) -> usize {
        let mut i = buddy_index;
        let mut possible_allocation = self.buddy_free_lists.elem_ref(i).unwrap().pointer().value();
        
        while i > 0 && allocation_size != BuddyAllocator::block_size_from_index(i) {            
            let (left, right) = self.split_buddy(i);
            let mut lower_level_allocator = self.buddy_free_lists_allocators.elem_ref_mut(i - 1);
            let buddy_lower_level = self.buddy_free_lists.elem_ref(i - 1);            
            let new_buddy_lower_level = buddy_lower_level.map(|e| e.pointer().add(right, &mut lower_level_allocator))
                                                         .or_else(|| Some(FreeList::new(right, &mut lower_level_allocator)));

            self.buddy_free_lists.update(i - 1, new_buddy_lower_level);

            let buddy_bitmap = self.buddy_bitmaps.elem_ref_mut(i - 1);
            buddy_bitmap.set_in_use(Frame::from_address(possible_allocation).number());
            
            possible_allocation = left;
            i -= 1;
        }

        possible_allocation
    }

    fn split_buddy(&self, buddy_index : usize) -> (usize, usize) {
        self.buddy_free_lists.elem_ref(buddy_index)
                             .map(|e| { 
                                    let left_address  = e.pointer().value();
                                    let block_size    = BuddyAllocator::block_size_from_index(buddy_index);
                                    let right_address = left_address + block_size / 2;

                                    (left_address, right_address)
                              })
                             .unwrap()
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
            let free_list_search_result = self.buddy_free_lists.elem_ref(buddy_index);                                                                                             
            
            if free_list_search_result.is_none() {
                let (buddy_index, free_list_value) = self.search_free_list_up(buddy_index + 1).unwrap();
                let result = self.split(allocation_size_rounded, buddy_index);

                Some(result)
            }
            else {
                let r = free_list_search_result.unwrap();
                Some(1)
            }
        }        
    }

    fn free(&mut self, pointer : usize) {

    }

    fn start_address(&self) -> usize {
        1
    }

    fn end_address(&self) -> usize {
        1
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