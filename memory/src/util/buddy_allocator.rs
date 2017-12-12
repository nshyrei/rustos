use allocator::MemoryAllocator;
use util::array::Array;
use util::frame_bitmap::FrameBitMap;
use util::free_list::FreeList;
use frame::{Frame, FRAME_SIZE};
use stdx::ptr::Unique;
use core::iter;

pub struct BuddyAllocator {
    allocation_sizes : Array<usize>,
    buddy_bitmaps    : Array<FrameBitMap>,
    buddy_free_lists : Array<Option<Unique<FreeList<Frame>>>>,
}

impl BuddyAllocator {
    pub unsafe fn new(start_address : usize, total_memory : usize, memory_allocator : &mut MemoryAllocator) -> BuddyAllocator {
        let total_frames_count = total_memory / FRAME_SIZE;        
        let total_buddy_levels = BuddyLevelSizesIterator::new(total_frames_count).count();
        let allocation_sizes      = Array::<usize>::new_fill_default(total_frames_count, memory_allocator);
        let mut buddy_bitmaps     = Array::<FrameBitMap>::new(total_buddy_levels, memory_allocator);
        let mut buddy_free_lists  = Array::<Option<Unique<FreeList<Frame>>>>::new_fill_default(total_buddy_levels, memory_allocator);


        let buddy_bitmap_mid = total_buddy_levels / 2;
        let last_bitmap_index = total_buddy_levels - 1;
        let mut buddy_level_sizes_it = BuddyLevelSizesIterator::new(total_frames_count);

        for i in 0..buddy_bitmap_mid + 1 { // mid inclusive
            let (sizeLower, sizeUpper) = buddy_level_sizes_it.next().unwrap();

            let buddy_bitmap_lower = FrameBitMap::new(sizeLower, memory_allocator);
            let buddy_bitmap_upper = FrameBitMap::new(sizeUpper, memory_allocator);
            buddy_bitmaps[i] = buddy_bitmap_lower;
            buddy_bitmaps[last_bitmap_index - i] = buddy_bitmap_upper;
        }

        let top_level_free_list = FreeList::new(Frame::from_address(start_address), memory_allocator);
        buddy_free_lists[last_bitmap_index] = Some(top_level_free_list);

        BuddyAllocator {
            allocation_sizes : allocation_sizes,
            buddy_bitmaps    : buddy_bitmaps,
            buddy_free_lists : buddy_free_lists
        }
    }
}

impl MemoryAllocator for BuddyAllocator {

    fn allocate(&mut self, size : usize) -> Option<usize> {

    }

    fn free(&mut self, pointer : usize){

    }
}

struct BuddyLevelSizesIterator {
    total_frames_count : usize,
    i : usize,    
}

impl BuddyLevelSizesIterator {
    fn new(total_frames_count : usize) -> Self {
        BuddyLevelSizesIterator {
            total_frames_count : total_frames_count,
            i : 1
        }
    }
}

impl iter::Iterator for BuddyLevelSizesIterator {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {

        
        let result = if self.i == 1 
        {            
            Some((self.total_frames_count, 1 as usize))
        }
        else if self.i * self.i < self.total_frames_count &&  
                self.total_frames_count % self.i == 0
        {
            Some((self.i, self.total_frames_count / self.i))
        }
        else if self.i * self.i == self.total_frames_count 
        {
            Some((self.i, self.i))
        }
        else 
        {
            None
        };

        self.i += 1;
        result 
    }
}

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

