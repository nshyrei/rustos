use allocator::MemoryAllocator;
use util::array::Array;
use util::frame_bitmap::FrameBitMap;
use util::free_list::FreeList;
use util::bump_allocator::BumpAllocator;
use frame::{Frame, FRAME_SIZE};
use stdx::ptr::Unique;
use stdx::iterator::IteratorExt;
use stdx::math;
use core::iter;
use core::mem;

pub struct BuddyAllocator {
    allocation_sizes : Array<usize>,
    buddy_bitmaps    : Array<FrameBitMap>,
    buddy_free_lists : Array<Option<Unique<FreeList<Frame>>>>,
    total_memory     : usize,
    start_address : usize,
    end_address : usize,
    memory_allocator : BumpAllocator
}

impl BuddyAllocator {
    pub unsafe fn new(start_address : usize, end_address : usize) -> BuddyAllocator {        
        let memory_size = end_address - start_address;
        let total_memory = Frame::aligned_down(memory_size).end_address();
        let total_frames_count = total_memory / FRAME_SIZE;        
        let total_buddy_levels = BuddyAllocator::buddy_index(total_memory);
        
        let sizes_array_size = mem::size_of::<usize>() * total_frames_count;
        let (buddy_bitmaps_size, buddy_free_lists_size) = BuddyAllocator::buddy_bitmaps_size(total_buddy_levels, total_frames_count);
        
        let aux_data_structures_size = sizes_array_size + buddy_bitmaps_size + buddy_free_lists_size; 

        let mut memory_allocator  = BumpAllocator::from_address(start_address, aux_data_structures_size);
        
        let allocation_sizes      = Array::<usize>::new(total_buddy_levels, &mut memory_allocator);
        let mut buddy_bitmaps     = Array::<FrameBitMap>::new(total_buddy_levels, &mut memory_allocator);
        let mut buddy_free_lists  = Array::<Option<Unique<FreeList<Frame>>>>::new(total_buddy_levels, &mut memory_allocator);

        buddy_free_lists.fill_default();

        let mut block_size = FRAME_SIZE;
        for i in 0 .. total_buddy_levels {
            let block_count = total_memory / block_size;
            let bitmap = FrameBitMap::new(block_count, &mut memory_allocator);

            buddy_bitmaps.update(i, bitmap);
            block_size *= 2;
        }        

        let top_level_free_list = FreeList::new(Frame::from_address(start_address), &mut memory_allocator);
        buddy_free_lists[total_buddy_levels - 1] = Some(top_level_free_list);

        BuddyAllocator {
            allocation_sizes : allocation_sizes,
            buddy_bitmaps    : buddy_bitmaps,
            buddy_free_lists : buddy_free_lists,
            total_memory     : total_memory,
            start_address : start_address,
            end_address : end_address,
            memory_allocator : memory_allocator
        }
    }

    fn buddy_bitmaps_size(buddy_levels_count : usize, total_memory : usize) -> (usize, usize) {

        let mut bitmaps_size = 0;
        let mut free_list_size = 0;
        let mut block_size = FRAME_SIZE;

        for _ in 0 .. buddy_levels_count {
            let block_count = total_memory / block_size;
            let free_list_cell_size = mem::size_of::<FreeList<Frame>>() + mem::size_of::<Option<Unique<FreeList<Frame>>>>() + mem::size_of::<Unique<FreeList<Frame>>>();
            bitmaps_size += FrameBitMap::cell_size(block_count) + mem::size_of::<FrameBitMap>();
            free_list_size += block_count * free_list_cell_size;
            block_size *= 2;
        }
        
        (bitmaps_size, free_list_size)
    } 

    fn round_to_nearest_block_size(allocation_size : usize) -> usize {
        if allocation_size % FRAME_SIZE != 0 {
            (allocation_size / FRAME_SIZE) + 1
        }
        else {
            allocation_size / FRAME_SIZE
        }
    }

    fn buddy_index(block_size : usize) -> usize {
        math::log2(block_size) - 12 // 2 ^ 12 = 4096 = FRAME_SIZE
    }

    fn search_free_list_up(&self, buddy_index : usize) -> Option<(usize, Unique<FreeList<Frame>>)> {
        let list_length = self.buddy_free_lists.length();
        if buddy_index + 1 < list_length {
            let mut result : Option<Unique<FreeList<Frame>>> = None;
            let mut i = buddy_index + 1;

            while i < list_length && result.is_none() {
                result = self.buddy_free_lists.elem_val(i);
                i += 1;
            }

            result.map(|e| (i - 1, e))
        }
        else {
            None
        }        
    }

    fn block_size_from_index(buddy_index : usize) -> usize {
        (2 as usize).pow((12 + buddy_index + 1) as u32)
    }

    fn split(&mut self, allocation_size : usize, buddy_index : usize) -> Frame {
        let mut i = buddy_index;
        let mut possible_allocation = self.buddy_free_lists.elem_val(i).unwrap().pointer().value_copy();
        
        while i > 0 && allocation_size != possible_allocation.number() {            
            let (left, right) = self.split_buddy(i);
            let buddy_lower_level = self.buddy_free_lists.elem_val(i - 1);            
            let new_buddy_lower_level = buddy_lower_level.map(|e| e.pointer().add(right, &mut self.memory_allocator))
                                                         .or_else(|| Some(FreeList::new(right, &mut self.memory_allocator)));

            self.buddy_free_lists.update(i - 1, new_buddy_lower_level);             

            let buddy_bitmap = self.buddy_bitmaps.elem_ref_mut(i - 1);
            buddy_bitmap.set_in_use(possible_allocation.number());
            
            possible_allocation = left;
            i -= 1;
        }

        possible_allocation                
    }

    fn split_buddy(&self, buddy_index : usize) -> (Frame, Frame) {
        self.buddy_free_lists.elem_val(buddy_index)
                             .map(|e| { 
                                    let start_frame = e.pointer().value_copy();
                                    let block_size = BuddyAllocator::block_size_from_index(buddy_index);

                                    (start_frame, Frame::from_address(block_size / 2))
                              })
                             .unwrap()
    }
}

impl MemoryAllocator for BuddyAllocator {

    fn allocate(&mut self, size : usize) -> Option<usize> {

        let total_buddy_levels = self.buddy_free_lists.length();
        let allocation_size_rounded = BuddyAllocator::round_to_nearest_block_size(size);

        if allocation_size_rounded > self.total_memory {
            None
        }
        else {
            // search free list that corresponds to 'closest_block_size'
            // if there are no entries search free list array upwards
            // to find bigger block to split
            let buddy_index = BuddyAllocator::buddy_index(allocation_size_rounded);            
            
            let free_list_opt = self.buddy_free_lists.elem_val(buddy_index)
                                                     .map(|e| (buddy_index, e))
                                                     .or_else(|| self.search_free_list_up(buddy_index));                                                     

            if free_list_opt.is_none() {
                None
            }
            else {
                let (buddy_index, free_list_value) = free_list_opt.unwrap();
                let result = self.split(allocation_size_rounded, buddy_index);


                Some(result.number())
            }
        }        
    }

    fn free(&mut self, pointer : usize) {

    }

    fn start_address(&self) -> usize {
        self.start_address
    }

    fn end_address(&self) -> usize {
        self.end_address
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