use stdx_memory::MemoryAllocator;
use stdx_memory::ConstantSizeMemoryAllocator;
use stdx_memory::collections::array::Array;
use stdx_memory::collections::double_linked_list::{BuddyMap, UsizeLinkedMap};
use stdx_memory::collections::frame_bitmap::FrameBitMap;
use stdx_memory::collections::linked_list::LinkedList;
use allocator::bump;
use allocator::free_list;
use frame::{Frame, FRAME_SIZE};
use stdx::iterator::IteratorExt;
use stdx::math;
use stdx::Sequence;
use stdx_memory::trees::avl;
use core::cmp;
use core::mem;
//use stdx_memory::collections::immutable::double_linked_list::DoubleLinkedListCell;

/*
macro_rules! block_sizes {
    ($total_buddy_levels:expr, $starting_block_size:expr) => {{
        (0 .. $total_buddy_levels).scan($starting_block_size, |block_size, _| {
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
    }}
}

const MIN_ALLOCATION_SIZE : usize = 32; //bytes

pub struct SlabAllocator {
    size_to_slab         : Array<Option<Slab>>,
    address_to_size      : avl::AVLTree<usize>,
    total_memory         : usize,
    start_address        : usize,
    array_allocator : bump::BumpAllocator,
    tree_allocator : free_list::FreeListAllocator,
    linked_list_allocator : free_list::FreeListAllocator 
}

impl cmp::Ord for free_list::FreeListAllocator {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.start_address().cmp(&other.start_address())
    }
}

impl cmp::PartialOrd for free_list::FreeListAllocator {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.start_address().partial_cmp(&other.start_address())
    }
}

impl cmp::Eq for free_list::FreeListAllocator {

}

impl cmp::PartialEq for free_list::FreeListAllocator {
    fn eq(&self, other: &Self) -> bool {
        self.start_address() == other.start_address()
    }
}

struct SlabKey {
    address : usize,
    allocator : free_list::FreeListAllocator
}

impl SlabKey {
    fn address(&self) -> usize {
        self.address
    }
}

struct Slab {
    // free data structures
    allocators : avl::AVLTree<DoubleLinkedListCell<free_list::FreeListAllocator>>,
    // allocate data structures
    non_full : DoubleLinkedListCell<free_list::FreeListAllocator>,
}

impl SlabAllocator {
    pub fn new(start_address1 : usize, end_address1 : usize) -> Self {
        let start_address      = Frame::address_align_up(start_address1);
        let end_address        = Frame::address_align_down(end_address1);
        let total_memory       = end_address - start_address + 1;

        assert!(total_memory >= MIN_ALLOCATION_SIZE, 
            "Cannot create allocator when total memory size < MIN_ALLOCATION_SIZE (32). Start address : {}, end address {}",
            start_address,
            end_address);

        let total_slab_count = SlabAllocator::total_slab_count(total_memory);
        let array_size = Array::<Option<Slab>>::mem_size_for_array(total_slab_count);
        let (avl_tree_size, linked_list_size) = SlabAllocator::buddy_free_list_size(total_slab_count, total_memory);
        let avl_tree_cell_size = avl::AVLTree::<free_list::FreeListAllocator>::cell_size();
        let linked_list_cell_size = mem::size_of::<LinkedList<free_list::FreeListAllocator>>();

        let mut array_allocator = bump::BumpAllocator::from_address(start_address, array_size);
        let mut tree_allocator = free_list::FreeListAllocator::from_address(
            array_allocator.end_address() + 1,
            avl_tree_size,
            avl_tree_cell_size);
        let mut linked_list_allocator = free_list::FreeListAllocator::from_address(
            tree_allocator.end_address() + 1,
            linked_list_size,
            linked_list_cell_size);
        
    }

    fn buddy_free_list_size(buddy_levels_count : usize, total_memory : usize) -> (usize, usize) {
        let mut tree_size = 0;
        let mut linked_list_size = 0;        

        for block_count in block_count!(total_memory, buddy_levels_count, MIN_ALLOCATION_SIZE) {
            tree_size += avl::AVLTree::<free_list::FreeListAllocator>::cell_size();
            linked_list_size += mem::size_of::<LinkedList<free_list::FreeListAllocator>>();
        }
        
        (tree_size, linked_list_size)
    }

    fn total_slab_count(total_memory : usize) -> usize {
        let idx = SlabAllocator::index_from_size(total_memory);

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
}
*/