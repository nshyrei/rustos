/*use stdx_memory::MemoryAllocator;
use stdx_memory::ConstantSizeMemoryAllocator;
use stdx_memory::collections::array::Array;
use stdx_memory::collections::double_linked_list::{BuddyMap, UsizeLinkedMap};
use stdx_memory::collections::frame_bitmap::FrameBitMap;
use stdx_memory::collections::linked_list::LinkedList;
use allocator::bump;
use frame::{Frame, FRAME_SIZE};
use stdx::iterator::IteratorExt;
use allocator::free_list;
use stdx::math;
use stdx::Sequence;

const MIN_ALLOCATION_SIZE : usize = 32; //bytes

pub struct SlabAllocator {
    size_to_slab         : Array<Slab>,
    address_to_size      : Array<usize>,
    total_memory         : usize,
    start_address        : usize,
}

struct Slab {
    address_to_allocator : Array<free_list::FreeListAllocator>,
    non_full : LinkedList<free_list::FreeListAllocator>
}

impl SlabAllocator {
    pub fn new(start_address1 : usize, end_address1 : usize) -> Self {
        let start_address      = Frame::address_align_up(start_address1);
        let end_address        = Frame::address_align_down(end_address1);
        let total_memory       = end_address - start_address + 1;

        assert!(total_memory >= MIN_ALLOCATION_SIZE, 
            "Cannot create allocator total memory size < MIN_ALLOCATION_SIZE (32). Start address : {}, end address {}",
            start_address,
            end_address);

        let total_allocation_levels = SlabAllocator::total_buddy_levels(total_memory);
        
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

        let allocation_sizes            = Array::<usize>::new(total_frames_count, &mut array_allocator);        
        let mut buddy_free_lists_array  = Array::<BuddyFreeList>::new(total_buddy_levels, &mut array_allocator);   

        BuddyAllocator::create_buddy_free_lists(
            &mut buddy_free_lists_array, 
            &mut array_allocator, 
            &mut free_list_allocator, 
            total_memory, 
            total_buddy_levels);

        // set initial block that covers all memory as free
        let idx = if total_buddy_levels > 0 { total_buddy_levels - 1} else { 0 };
        buddy_free_lists_array[idx].set_free(0, &mut free_list_allocator);
                
        BuddyAllocator {
            allocation_sizes            : allocation_sizes,            
            buddy_free_lists            : buddy_free_lists_array,            
            total_memory                : total_memory,            
            array_allocator             : array_allocator,
            free_list_allocator         : free_list_allocator,
            start_address               : start_address,            
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

    fn block_size_from_index(buddy_index : usize) -> usize {
        // 2 ^ 5 = 32 = MIN_ALLOCATION_SIZE        
        1 << (5 + buddy_index)
    }

    fn index_from_size(block_size : usize) -> usize {
        let log = math::log2_align_down(block_size);
        if log < 5 {
            0
        }
        else {
            log - 5 // 2 ^ 5 = 32 = MIN_ALLOCATION_SIZE
        }        
    }
}*/