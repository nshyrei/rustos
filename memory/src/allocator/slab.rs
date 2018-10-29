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
use stdx_memory::collections::immutable::double_linked_list::DoubleLinkedList;
use stdx_memory::heap::RC;
use stdx_memory::heap::Box;
use stdx_memory::heap::WeakBox;
use allocator::buddy::BuddyAllocator;
use core::alloc::Alloc;
use core::alloc::Layout;
use core::alloc::AllocErr;
use core::ptr::NonNull;
use core::ops::DerefMut;
use core::ops::Deref;
use self::free_list::FreeListAllocator;

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

type ListOfAllocators = DoubleLinkedList<free_list::FreeListAllocator, free_list::FreeListAllocator>;
type PtrToListOfAllocators = RC<ListOfAllocators, free_list::FreeListAllocator>;
type ProperPtr = RC<Box<ListOfAllocators, free_list::FreeListAllocator>, free_list::FreeListAllocator>;

struct Slab {
    // free data structures
    non_empty : avl::AVLTree<ProperPtr>,
    // allocate data structures
    non_full : Option<ProperPtr>
}

impl Slab {
    fn new(allocator : free_list::FreeListAllocator, memory_allocator : &mut free_list::FreeListAllocator) -> Self {
        let cell = RC::new(DoubleLinkedList::new(allocator, memory_allocator).promote(memory_allocator), memory_allocator);

        Slab {
            non_empty : avl::AVLTree::new(),
            non_full  : Some(cell)
        }
    }

    // slab size is passed here to prevent saving it in slab structure, because slab allocator
    // knows what slabs and of what sizes it has

    fn allocate(&mut self, size : usize, slab_size : usize, frame_allocator : &mut BuddyAllocator, memory_allocator : &mut free_list::FreeListAllocator) -> Option<usize> {

        let non_full_allocation_result = self.try_allocate_non_full(size, slab_size, frame_allocator, memory_allocator);

        if  non_full_allocation_result.is_some() {
            non_full_allocation_result
        }
        else {
            if let Some(new_frame) = frame_allocator.allocate(slab_size) {
                let allocator = free_list::FreeListAllocator::from_address(new_frame, slab_size, size);
                let mut cell = RC::new(DoubleLinkedList::new(allocator, memory_allocator).promote(memory_allocator), memory_allocator);
                let result = cell.value_mut().allocate_size();

                self.non_empty.insert(RC::clone(&cell), memory_allocator);
                self.non_full = Some(cell);

                result
            }
            else {
                None
            }
        }
    }

    fn try_allocate_non_full(&mut self, size : usize, slab_size : usize, frame_allocator : &mut BuddyAllocator, memory_allocator : &mut free_list::FreeListAllocator) -> Option<usize> {
        if self.non_full.is_none() {
            None
        }
        else {
            let mut allocator = self.non_full.take().unwrap();
            let allocation_result = allocator.value_mut().allocate_size();

            if allocation_result.is_some() {
                self.non_full = Some(allocator);
                allocation_result
            }
            else {
                let prev = allocator.prev().as_ref().map(|p| {
                    let promoted = WeakBox::from_pointer(p).leak().promote(memory_allocator);

                    RC::new(promoted, memory_allocator)
                });

                self.non_full = prev;

                DoubleLinkedList::modify_neighbour_connections(allocator);

                // try allocate for prev
                self.try_allocate_non_full(size, slab_size, frame_allocator, memory_allocator)
            }
        }
    }

    fn free(&mut self, pointer : usize, memory_allocator : &mut free_list::FreeListAllocator) {

        let mut allocator_is_free = false;
        if let Some(ref mut non_full_dlist) = self.non_full {
            let mut allocator = non_full_dlist.value_mut();

            if allocator.is_inside_address_space(pointer) {
                allocator.free_size(pointer);
            }

            allocator_is_free = allocator.fully_free();
        }

        if allocator_is_free && self.non_full.is_some() {
            let mut head = self.non_full.take().unwrap();
            let head_start_addr = head.value().start_address();
            {
                let prev = head.prev().as_ref().map(|p| {
                    let promoted = WeakBox::from_pointer(p).leak().promote(memory_allocator);

                    RC::new(promoted, memory_allocator)
                });

                self.non_full = prev;
            }

            DoubleLinkedList::modify_neighbour_connections(head);

            self.non_empty.delete_by(head_start_addr, |n| n.value().start_address());

            return;
        }

        let mut dlist_opt = self.non_empty.find_by(&pointer,
                                               |node| node.value().start_address(),
                                               |node, ptr| node.value().is_inside_address_space(*ptr));

        let mut allocator_is_free_tree_case = false;


        if let Some(mut dlist) = dlist_opt.as_mut() {
            let mut allocator = dlist.value_mut();
            allocator.free(pointer);

            allocator_is_free_tree_case = allocator.fully_free();
        }

        if allocator_is_free_tree_case && dlist_opt.is_some() {
            let dlist = dlist_opt.unwrap();
            let start_address = dlist.value().start_address();

            DoubleLinkedList::modify_neighbour_connections(dlist.leak());

            self.non_empty.delete_by(start_address, |n| n.value().start_address());
        }
    }
}

pub struct SlabAllocator {
    size_to_slab         : Array<Option<Slab>>,
    address_to_size      : avl::AVLTree<usize>,
    total_memory         : usize,
    start_address        : usize,
    array_allocator      : bump::BumpAllocator,
    tree_allocator       : free_list::FreeListAllocator,
    linked_list_allocator : free_list::FreeListAllocator
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
        let array_size = Array::<Option<Slab>>::mem_size_for(total_slab_count);
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

        let size_to_slab = Array::<Option<Slab>>::new(total_slab_count, &mut array_allocator);
        let address_to_size = avl::AVLTree::<usize>::new();

        SlabAllocator {
            size_to_slab,
            address_to_size,
            total_memory,
            start_address,
            array_allocator,
            tree_allocator,
            linked_list_allocator
        }
        
    }

    fn new_slab(&mut self)  {

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

    fn allocation_size_rounded(size : usize) -> usize {
        let allocation_size_rounded0 = (2 as usize).pow(math::log2_align_up(size) as u32);

        if allocation_size_rounded0 < MIN_ALLOCATION_SIZE {
            MIN_ALLOCATION_SIZE
        } else {
            allocation_size_rounded0
        }
    }
}

impl MemoryAllocator for SlabAllocator {

    fn allocate(&mut self, size: usize) -> Option<usize> {

        if size == 0 {
            None
        }
        else {
            let size_rounded = SlabAllocator::allocation_size_rounded(size);

            if size_rounded > self.total_memory {
                None
            }
            else {
                let size_array_idx = SlabAllocator::index_from_size(size_rounded);
                let mut slab = self.size_to_slab[size_array_idx];

                if slab.is_some() {
                    slab.as_mut()
                }

            }


        }
    }

    fn free(&mut self, pointer: usize) {
        unimplemented!()
    }
}

unsafe impl Alloc for SlabAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let size = layout.size();
        if size == 0 {
            Err(AllocErr {})
        } else {

            let allocation_size_rounded = SlabAllocator::allocation_size_rounded(size);

            if allocation_size_rounded > self.total_memory {
                Err(AllocErr {})
            } else {

                let slab_opt = &self.size_to_slab[(allocation_size_rounded / MIN_ALLOCATION_SIZE) - 1].as_mut();

                if let Some(slab) = slab_opt {

                }


                unsafe { Ok(NonNull::new_unchecked(0 as *mut _)) }
            }
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        unimplemented!()
    }
}