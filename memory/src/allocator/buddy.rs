use stdx_memory::MemoryAllocator;
use stdx_memory::ConstantSizeMemoryAllocator;
use stdx_memory::collections::array::Array;
use stdx_memory::collections::double_linked_list::BuddyMap;
use stdx_memory::heap;
use allocator::bump;
use frame::{Frame, FRAME_SIZE};
use stdx::iterator::IteratorExt;
use allocator::free_list;
use stdx::math;
use stdx::Sequence;
use core::iter;
use core::mem;
use core::ops;
use display::vga::writer;
use core::fmt::Write;
use util::frame_bitmap::FrameBitMap;


macro_rules! block_sizes {
    ($total_buddy_levels:expr, $starting_block_size:expr) => {{
        (0 .. $total_buddy_levels).scan($starting_block_size, |block_size, x| {
            let result = *block_size;
            *block_size = *block_size * 2;

            Some(result)
        })
    }}
}

macro_rules! block_count {
    ($total_memory:expr, $total_buddy_levels:expr, $starting_block_size:expr) => {{
        block_sizes!($total_buddy_levels, $starting_block_size)
            .map(|block_size| $total_memory / block_size)
            .index_items()
    }}
}

pub struct BuddyAllocator {
    allocation_sizes     : Array<usize>,    
    buddy_free_lists     : Array<BuddyFreeList>,    
    array_allocator      : bump::BumpAllocator,
    free_list_allocator  : free_list::FreeListAllocator,
    total_memory         : usize,
    start_address        : usize,
    merge_status         : FrameBitMap
}

impl BuddyAllocator {

    fn start_address(&self) -> usize {
        1
    }

    fn end_address(&self) -> usize {
        1
    }

    pub fn new(start_address1 : usize, end_address1 : usize) -> BuddyAllocator {
        let start_address      = Frame::address_align_up(start_address1);
        let end_address        = end_address1;

        assert!(end_address > start_address, "Cannot create allocator when end address <= start address");

        let total_memory       = end_address - start_address + 1;

        let total_frames_count = Frame::from_address(total_memory).number();

        assert!(end_address > start_address, "Cannot create allocator when total memory size < FRAME_SIZE (4096)");

        let total_buddy_levels = BuddyAllocator::total_buddy_levels(total_memory);
        
        let sizes_array_size           = Array::<usize>::mem_size_for(total_frames_count);
        let bitmap_size                = FrameBitMap::mem_size_for(total_frames_count);
        let buddy_free_list_array_size = Array::<BuddyFreeList>::mem_size_for(total_buddy_levels);
        let (buddy_array_size, buddy_free_lists_size) = BuddyAllocator::buddy_free_list_size(
            total_buddy_levels,
            total_memory);
        
        let array_sizes = sizes_array_size + buddy_array_size + buddy_free_list_array_size + bitmap_size;

        let mut array_allocator     = bump::BumpAllocator::from_address(start_address, array_sizes);
        let mut free_list_allocator = free_list::FreeListAllocator::from_address(
            array_allocator.end_address() + 1, 
            buddy_free_lists_size,
            BuddyMap::cell_size());

        let allocation_sizes            = Array::<usize>::new(total_buddy_levels, &mut array_allocator);
        let merge_status                = FrameBitMap::new(total_frames_count, &mut array_allocator);        
        let mut buddy_free_lists_array  = Array::<BuddyFreeList>::new(total_buddy_levels, &mut array_allocator);        

        BuddyAllocator::create_buddy_free_lists(
            &mut buddy_free_lists_array, 
            &mut array_allocator, 
            &mut free_list_allocator, 
            total_memory, 
            total_buddy_levels);

        // set initial block that covers all memory as free
        let idx = if total_buddy_levels > 0 { total_buddy_levels - 1} else { 0 };
        buddy_free_lists_array[idx].buddy_map().set_free(0, &mut free_list_allocator);
                
        BuddyAllocator {
            allocation_sizes            : allocation_sizes,            
            buddy_free_lists            : buddy_free_lists_array,            
            total_memory                : total_memory,            
            array_allocator             : array_allocator,
            free_list_allocator         : free_list_allocator,
            start_address               : start_address,
            merge_status                : merge_status
        }
    }

    fn total_buddy_levels(total_memory : usize) -> usize {
        let idx = BuddyAllocator::index_from_size(total_memory);

        if idx > 0 {
            idx + 1
        }
        else {
            1
        }
    }

    fn create_buddy_free_lists(buddy_free_lists : &mut Array<BuddyFreeList>, 
        array_allocator : &mut bump::BumpAllocator,
        free_list_allocator : &mut free_list::FreeListAllocator,
        total_memory : usize, 
        total_buddy_levels : usize)
    {
        let it = block_count!(total_memory, total_buddy_levels, FRAME_SIZE);//BlockCountIterator::new(total_memory, total_buddy_levels, FRAME_SIZE).index_items();

        for (block_count, i) in it {
            let buddy_free_list = BuddyMap::new(block_count, array_allocator, free_list_allocator);            
            //buddy_free_lists.update(i, BuddyFreeList(buddy_free_list)); 
        }
    }

    fn buddy_free_list_size(buddy_levels_count : usize, total_memory : usize) -> (usize, usize) {
        let mut array_size = 0;
        let mut free_list_size = 0;        

        for (block_count, _) in block_count!(total_memory, buddy_levels_count, FRAME_SIZE) {            
            free_list_size += BuddyMap::mem_size_for_linked_list(block_count);
            array_size += BuddyMap::mem_size_for_array(block_count);            
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
            else if self.buddy_free_lists[i].buddy_map_ref().has_free_block() {
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
        let log = math::log2_align_down(block_size);
        if log < 12 {
            0
        }
        else {
            log - 12 // 2 ^ 12 = 4096 = FRAME_SIZE
        }        
    }    

    fn address_to_index(address : usize, buddy_list_index : usize) -> usize {                    
        address / BuddyAllocator::block_size_from_index(buddy_list_index)
    }

    fn split_down(&mut self, allocation_size : usize, start_index : usize) -> Option<(usize, isize, usize)> {
        let mut i = start_index as isize;

        loop {
            
            if i < 0 {
                return None
            }

            let current_level_size = BuddyAllocator::block_size_from_index(i as usize);            
            let left = self.buddy_free_lists[i].buddy_map().first_free_block(&mut self.free_list_allocator).unwrap();

            // we can return current block or split it at that point,
            // both operations will set the block to 'in use'
            self.buddy_free_lists[i].buddy_map().set_in_use(left, &mut self.free_list_allocator);

            if allocation_size == current_level_size {
                return Some((left, i, left * current_level_size))
            }
            else {
                // split buddy
                let left_start_address     = left * current_level_size;
                let lower_level_size       = BuddyAllocator::block_size_from_index((i - 1) as usize);
                let right_start_address    = left_start_address + lower_level_size;
                let right                  = right_start_address / lower_level_size;       

                // important to set left before right to in use, for it to appear
                // on top of the free blocks stack. The reason for this is that
                // because of allocator convention, e.g. picking left blocks first.
                self.buddy_free_lists[i - 1].buddy_map().set_free(left, &mut self.free_list_allocator);               

                self.buddy_free_lists[i - 1].buddy_map().set_free(right, &mut self.free_list_allocator);                                            
                                
                i -= 1;
            }
        }
    }

    fn merge_up(&mut self, pointer : usize, start_index : usize) {        
        let buddy_lists_count    = self.buddy_free_lists.length();

        // index across buddy list array
        let mut buddy_list_index = start_index;
        let mut block_address    = pointer;

        loop {
            if buddy_list_index > buddy_lists_count - 1 {
                break;
            }

            let buddy_free_list            = &mut self.buddy_free_lists[buddy_list_index];            
            let block_index                = BuddyAllocator::address_to_index(block_address, buddy_list_index);
            let buddy_start_address        = buddy_free_list.buddy_start_address(block_index, block_address);

            let block_frame_number = Frame::number_for_address(block_address);
            let buddy_frame_number = Frame::number_for_address(buddy_start_address);
            let block_not_merging = self.merge_status.is_free(block_frame_number);
            let buddy_not_merging = self.merge_status.is_free(buddy_frame_number);

            let freed_block_is_in_use = buddy_free_list.buddy_map().is_in_use(block_index);
            let buddy_is_free         = buddy_free_list.buddy_map().is_free(BuddyFreeList::buddy_index(block_index));

            // if current block can be freed and buddy is also free and both doesn't have a merge status            
            // then we can perfrom a merge
            if freed_block_is_in_use 
                && buddy_is_free
                && block_not_merging
                && buddy_not_merging 
            {
                self.merge_status.set_in_use(block_frame_number);
                self.merge_status.set_in_use(buddy_frame_number);

                buddy_free_list.set_buddy_in_use(block_index, &mut self.free_list_allocator);
                buddy_list_index += 1;
            }

            // only current block can be freed
            else if freed_block_is_in_use && block_not_merging && !buddy_is_free {
                buddy_free_list.buddy_map().set_free(block_index, &mut self.free_list_allocator);
                break;
            }
            // current block is already free somehow -> do nothing
            else {
                break;
            }
        };
    }
}

impl MemoryAllocator for BuddyAllocator {

    fn allocate(&mut self, size : usize) -> Option<usize> {

        let allocation_size_rounded = (2 as usize).pow(math::log2_align_up(size) as u32);

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
                             .and_then(|index_with_free_block| 
                                self.split_down(allocation_size_rounded, index_with_free_block)
                             );
            
            if let Some((frame_number, new_buddy_index, result_address)) = result {
                
                self.allocation_sizes[frame_number] = new_buddy_index as usize;
                self.merge_status.set_free(frame_number);

                Some(result_address + self.start_address)
            }
            else {
                None
            }
        }
    }

    fn free(&mut self, pointer : usize) {
        let normalized_pointer = pointer - self.start_address;
        let frame_number       = Frame::number_for_address(normalized_pointer);

        if self.merge_status.is_free(frame_number) {
            let buddy_list_index   = self.allocation_sizes[frame_number];        

            self.merge_up(normalized_pointer, buddy_list_index);
        }        
    }
}



/*
type BlockSizeIterator = iter::Map<ops::Range<usize>, FnMut(usize) -> usize>;

struct BlockCountIterator {
    buddy_levels_range : ops::Range<usize>,
    block_size         : usize,    
    total_memory       : usize
}

impl BlockCountIterator {
    fn new(total_memory : usize, total_buddy_levels : usize, starting_block_size : usize) -> Self {
        BlockCountIterator {
            buddy_levels_range : (0..total_buddy_levels),            
            block_size         : starting_block_size,            
            total_memory       : total_memory
        }
    }
}

impl iter::Iterator for BlockCountIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {

        if self.buddy_levels_range.next().is_some() {
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
*/
struct BuddyFreeList {
    buddy_map  : BuddyMap,
    block_size : usize
}

impl BuddyFreeList {

    pub fn new<A, B>(length : usize, block_size : usize, memory_allocator : &mut A, list_allocator : &mut B) -> Self 
    where A : MemoryAllocator, B : ConstantSizeMemoryAllocator {
        BuddyFreeList {
            buddy_map  : BuddyMap::new(length, memory_allocator, list_allocator),
            block_size : block_size
        }
    }

    fn buddy_index(i : usize) -> usize {        
        if math::is_even(i) {
            i + 1
        }
        else {
            i - 1
        }
    }

    fn is_left_buddy(i : usize) -> bool {
        math::is_even(i)
    }

    fn is_right_buddy(i : usize) -> bool {
        !BuddyFreeList::is_left_buddy(i)
    }

    fn buddy_start_address(&self, i : usize, address : usize) -> usize {
        if math::is_even(i) {
            address + self.block_size
        }
        else {
            address - self.block_size
        }
    }

    fn buddy_map_ref(&self) -> &BuddyMap {
        &self.buddy_map
    }

    fn buddy_map(&mut self) -> &mut BuddyMap {
        &mut self.buddy_map
    }

    fn set_buddy_in_use<A>(&mut self, index : usize, memory_allocator : &mut A)
    where A : MemoryAllocator {
        let buddy_index = BuddyFreeList::buddy_index(index);
        self.buddy_map.set_in_use(buddy_index, memory_allocator);
    }
}