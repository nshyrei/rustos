use allocator::MemoryAllocator;
use util::array::Array;
use util::frame_bitmap::FrameBitMap;
use util::free_list::FreeList;
use frame::{Frame, FRAME_SIZE};
use stdx::ptr::Unique;
use stdx::iterator::IteratorExt;
use core::iter;
use core::f64;

pub struct BuddyAllocator<'a> {
    allocation_sizes : Array<usize>,
    buddy_bitmaps    : Array<FrameBitMap>,
    buddy_free_lists : Array<Option<Unique<FreeList<Frame>>>>,
    total_memory     : usize,
    memory_allocator : &'a mut MemoryAllocator
}

impl<'a> BuddyAllocator<'a> {
    pub unsafe fn new(start_address : usize, total_memory : usize, memory_allocator : &mut MemoryAllocator) -> BuddyAllocator {        
        let total_frames_count = total_memory / FRAME_SIZE;        
        let total_buddy_levels = BuddyAllocator::log(FRAME_SIZE, total_memory);
        let allocation_sizes      = Array::<usize>::new_fill_default(total_frames_count, memory_allocator);
        let mut buddy_bitmaps     = Array::<FrameBitMap>::new(total_buddy_levels, memory_allocator);
        let mut buddy_free_lists  = Array::<Option<Unique<FreeList<Frame>>>>::new_fill_default(total_buddy_levels, memory_allocator);


        let mut buddy_level_sizes_it = BuddyLevelSizesIterator::new(total_frames_count);
        let mut i = 0;
        while let Some(buddy_bitmap_size) = buddy_level_sizes_it.next() {            
            buddy_bitmaps[i] = FrameBitMap::new(buddy_bitmap_size, memory_allocator);
            i += 1;
        };
        

        let top_level_free_list = FreeList::new(Frame::from_address(start_address), memory_allocator);
        buddy_free_lists[total_buddy_levels - 1] = Some(top_level_free_list);

        BuddyAllocator {
            allocation_sizes : allocation_sizes,
            buddy_bitmaps    : buddy_bitmaps,
            buddy_free_lists : buddy_free_lists,
            total_memory     : total_memory,
            memory_allocator : memory_allocator
        }
    }

    fn log(base : usize, x : usize) -> usize {        
        let mut result = 0;
        let mut pow = base;

        while pow <= x {
            pow *= 2;
            result += 1;
        }

        result
    }

    fn log2(x : usize) -> usize {
        BuddyAllocator::log(2, x)
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
        12 - BuddyAllocator::log2(block_size) // 2 ^ 12 = 4096 = FRAME_SIZE
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

    fn split(&mut self, allocation_size : usize, buddy_index : usize) {
        let mut i = buddy_index;
        let mut possible_allocation = self.buddy_free_lists.elem_val(i).unwrap().pointer().value_copy();
        
        while i > 0 && allocation_size != possible_allocation.number() {
            let mut splitted = self.split_buddy(i);
            let (left, right) = splitted;
            

            let buddy_lower_level = self.buddy_free_lists.elem_val(i - 1);
            let mut new_buddy_lower_level = buddy_lower_level.map(|e| e.pointer().add(right, self.memory_allocator));
            let mut r = self.buddy_free_lists.elem_ref_mut(1);
             r = &mut new_buddy_lower_level;


            let buddy_bitmap = self.buddy_bitmaps.elem_ref_mut(i - 1);
            buddy_bitmap.set_in_use(possible_allocation.number());
            
            possible_allocation = left;

            
            

            
        }
                
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

impl<'a> MemoryAllocator for BuddyAllocator<'a> {

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



                Some(1)
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

/*
fn total_buddy_levels(total_frame_count : usize) -> usize {
        let mut result = 0;
        let mut i = 1;

        while i * i < total_frame_count {
            if total_frame_count % i == 0 {
                result += 2;
            }

            i += 1;
        }

        if i * i == total_frame_count {
            result + 1
        }
        else {
            result
        }
    }
*/
