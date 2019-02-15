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
use stdx::Iterable;
use stdx::math;
use stdx::Sequence;
use stdx_memory::trees::avl;
use core::cmp;
use core::mem;
use stdx_memory::collections::immutable::double_linked_list::DoubleLinkedList;
use stdx_memory::heap::RC;
use stdx_memory::heap::Box;
use stdx_memory::heap::RCBox;
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

type ListOfAllocators = DoubleLinkedList<free_list::FreeListAllocator, free_list::FreeListAllocator>;
type ProperPtr             = RC<ListOfAllocators, free_list::FreeListAllocator>;

#[repr(C)]
struct Slab {
    // free data structures
    non_empty : avl::AVLTree<ProperPtr, free_list::FreeListAllocator>,
    // allocate data structures
    non_full : Option<ProperPtr>
}

impl Slab {
    fn new(allocation_size: usize, frame_allocator: &mut BuddyAllocator, memory_allocator: &mut free_list::FreeListAllocator) -> Option<Self> {
        frame_allocator.allocate(allocation_size).map(|slab_start_address| {

            let slab_size = BuddyAllocator::true_allocation_size_for(allocation_size);
            let allocator = free_list::FreeListAllocator::from_address(slab_start_address, slab_size, allocation_size);

            let allocator_boxed = DoubleLinkedList::new_rc(allocator, memory_allocator);

            let tree = avl::AVLTree::new(RC::clone(&allocator_boxed), memory_allocator);

            Slab {
                non_empty: tree,
                non_full: Some(allocator_boxed),
            }
        })
    }

    // slab size is passed here to prevent saving it in slab structure, because slab allocator
    // knows what slabs and of what sizes it has
    fn allocate(&mut self, size : usize, slab_size : usize, frame_allocator : &mut BuddyAllocator, memory_allocator : &mut free_list::FreeListAllocator) -> Option<usize> {
        let non_full_allocation_result = self.try_allocate_non_full(size, frame_allocator, memory_allocator);

        non_full_allocation_result.or_else(|| {
            // try increase slab size
            let true_slab_size                 = BuddyAllocator::true_allocation_size_for(slab_size);
            let frame_allocation_result = frame_allocator.allocate(true_slab_size);

            frame_allocation_result.and_then(|new_frame| {
                let mut allocator = free_list::FreeListAllocator::from_address(new_frame, true_slab_size, slab_size);

                let result = allocator.allocate_size(); // this will always succeed

                let cell = match self.non_full {
                    Some(ref mut node) => {
                        DoubleLinkedList::add(&mut RC::clone(node),  allocator, memory_allocator)},
                    _ =>
                        DoubleLinkedList::new_rc(allocator, memory_allocator)
                };

                self.non_empty.insert(RC::clone(&cell), memory_allocator);
                self.non_full = Some(cell);

                result
            })
        })
    }

    fn try_allocate_non_full(&mut self, size: usize, frame_allocator: &mut BuddyAllocator, memory_allocator: &mut free_list::FreeListAllocator) -> Option<usize> {
        self.non_full
            .as_mut()
            .and_then(|allocator_list| allocator_list.value_mut().allocate_size())
    }

    fn address_belongs_to_non_full(&self, pointer : usize) -> bool {
        self.non_full
                .as_ref()
                .map(|list_cell| list_cell.value().is_inside_address_space(pointer) )
                .unwrap_or(false)
    }

    fn free_from_non_full(&mut self, pointer : usize, memory_allocator : &mut free_list::FreeListAllocator, frame_allocator : &mut BuddyAllocator) {
        let mut allocator_is_free = false;

        if let Some(ref mut non_full_dlist) = self.non_full {
            let mut allocator = non_full_dlist.value_mut();

            allocator.free_size(pointer);

            allocator_is_free = allocator.fully_free();
        }

        if allocator_is_free {
            if let Some(mut head) = self.non_full.take() {
                let head_start_addr = head.value().start_address();

                {
                    let prev = head.prev();

                    self.non_full = prev;
                }

                DoubleLinkedList::modify_neighbour_connections(head);

                self.non_empty.delete_by(head_start_addr, |n| n.value().start_address());

                frame_allocator.free(head_start_addr);
            }
        }
    }

    fn free_from_non_empty(&mut self, pointer : usize, memory_allocator : &mut free_list::FreeListAllocator, frame_allocator : &mut BuddyAllocator) {

        let mut dlist_opt = self.non_empty.find_by(&pointer,
                                                   |node| node.value().start_address(),
                                                   |node, ptr| node.value().is_inside_address_space(*ptr));

        let mut allocator_is_free_tree_case = false;

        if let Some(mut dlist) = dlist_opt.as_mut() {
            let mut allocator = dlist.value_mut();
            allocator.free_size(pointer);

            allocator_is_free_tree_case = allocator.fully_free();
        }

        if allocator_is_free_tree_case {
            if let Some(dlist) = dlist_opt {
                let start_address = dlist.value().start_address();

                DoubleLinkedList::modify_neighbour_connections(dlist.leak());

                self.non_empty.delete_by(start_address, |n| n.value().start_address());

                frame_allocator.free(start_address);
            }
        }
    }

    fn free(&mut self, pointer : usize, memory_allocator : &mut free_list::FreeListAllocator, frame_allocator : &mut BuddyAllocator) {
        if  self.address_belongs_to_non_full(pointer)  {
            self.free_from_non_full(pointer, memory_allocator, frame_allocator);
        }
        else {
            // if the address doesn't belong to first non-free slab then search through list
            self.free_from_non_empty(pointer, memory_allocator, frame_allocator);
        }
    }

    fn is_fully_free(&self) -> bool {
        self.non_full.is_none() && self.non_empty.is_empty()
    }
}

pub struct SlabAllocator {
    size_to_slab                : Array<Option<Slab>>,
    address_to_size         : avl::AVLTree<(usize, usize), free_list::FreeListAllocator>,
    total_memory            : usize,
    start_address             : usize,
    array_allocator         : bump::BumpAllocator,
    tree_allocator            : free_list::FreeListAllocator,
    linked_list_allocator : free_list::FreeListAllocator,
    frame_allocator        : BuddyAllocator
}

impl SlabAllocator {

    pub fn is_fully_free(&self) ->bool {
        let slabs_are_empty = self.size_to_slab.iterator().all(|e| e.is_none());
        let tree_is_empty = self.address_to_size.is_empty();

         slabs_are_empty && tree_is_empty
    }

    pub fn new(start_address1 : usize, end_address1 : usize, frame_allocator : BuddyAllocator) -> Self {
        let start_address      = Frame::address_align_up(start_address1);
        let end_address        = Frame::address_align_down(end_address1);
        let total_memory      = end_address - start_address + 1;

        assert!(total_memory > MIN_ALLOCATION_SIZE,
                "Cannot create allocator when total memory size < MIN_ALLOCATION_SIZE (32). Start address : {}, end address {}",
                start_address,
                end_address);

        let total_slab_count                             = SlabAllocator::total_slab_count(total_memory);
        let array_size                                         = Array::<Option<Slab>>::mem_size_for(total_slab_count);
        let (avl_tree_size, linked_list_size)  = SlabAllocator::buddy_free_list_size(total_slab_count, total_memory);
        let avl_tree_cell_size                           = avl::AVLTree::<free_list::FreeListAllocator, free_list::FreeListAllocator>::cell_size();
        let linked_list_cell_size                       = mem::size_of::<DoubleLinkedList<free_list::FreeListAllocator, free_list::FreeListAllocator>>();

        let mut array_allocator             = bump::BumpAllocator::from_address(start_address, array_size);
        let mut tree_allocator               = free_list::FreeListAllocator::from_address(array_allocator.end_address() + 1, avl_tree_size, avl_tree_cell_size);
        let mut linked_list_allocator    = free_list::FreeListAllocator::from_address(tree_allocator.end_address() + 1, linked_list_size, linked_list_cell_size);

        let size_to_slab         = Array::<Option<Slab>>::new(total_slab_count, &mut array_allocator);
        let address_to_size = avl::AVLTree::<(usize, usize), free_list::FreeListAllocator>::new_empty();

        SlabAllocator {
            size_to_slab,
            address_to_size,
            total_memory,
            start_address,
            array_allocator,
            tree_allocator,
            linked_list_allocator,
            frame_allocator
        }

    }

    fn buddy_free_list_size(buddy_levels_count : usize, total_memory : usize) -> (usize, usize) {
        let mut tree_size = 0;
        let mut linked_list_size = 0;

        for block_count in block_count!(total_memory, buddy_levels_count, MIN_ALLOCATION_SIZE) {
            tree_size += avl::AVLTree::<free_list::FreeListAllocator, free_list::FreeListAllocator>::cell_size();
            linked_list_size += mem::size_of::<DoubleLinkedList<free_list::FreeListAllocator, free_list::FreeListAllocator>>();
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

    fn allocate0(
        size_rounded : usize,
        slab : &mut Slab,
        frame_allocator : &mut BuddyAllocator,
        linked_list_allocator :  &mut FreeListAllocator,
        tree_allocator : &mut FreeListAllocator,
        address_to_size : &mut avl::AVLTree<(usize, usize), free_list::FreeListAllocator>) -> Option<usize> {

        slab.allocate(
            size_rounded,
            size_rounded,
            frame_allocator,
            linked_list_allocator
        ).map(|r| {
            address_to_size.insert((r, size_rounded), tree_allocator);
            r
        })
    }

}

impl MemoryAllocator for SlabAllocator {
    fn allocate(&mut self, size: usize) -> Option<usize> {
        if size == 0 {
            None
        } else {
            let size_rounded = SlabAllocator::allocation_size_rounded(size);

            if size_rounded > self.total_memory {
                None
            } else {
                let size_array_idx = SlabAllocator::index_from_size(size_rounded);

                let mut frame_allocator            = &mut self.frame_allocator;
                let mut linked_list_allocator     = &mut self.linked_list_allocator;
                let mut tree_allocator               = &mut self.tree_allocator;
                let mut address_to_size            = &mut self.address_to_size;

                let result_from_existing_slab = {
                    let mut slab_opt = &mut self.size_to_slab[size_array_idx];

                    slab_opt.as_mut().and_then(|mut slab| {
                        SlabAllocator::allocate0(
                            size_rounded,
                            slab,
                            frame_allocator,
                            linked_list_allocator,
                            tree_allocator,
                            address_to_size)
                    })
                };

                let mut size_to_slab  = &mut self.size_to_slab;

                result_from_existing_slab.or_else(|| {
                    let new_slab_opt = Slab::new(size_rounded, frame_allocator, linked_list_allocator);

                    new_slab_opt.and_then(|mut new_slab| {
                        let result = SlabAllocator::allocate0(
                            size_rounded,
                            &mut new_slab,
                            frame_allocator,
                            linked_list_allocator,
                            tree_allocator,
                            address_to_size);

                        size_to_slab[size_array_idx] = Some(new_slab);

                        result
                    })
                })
            }
        }
    }

    fn free(&mut self, pointer: usize) {
        if let Some(was_allocated) = self.address_to_size.find_by(&pointer, |w| w.0,  |x, y| x .1== *y) {
            let slab_array_idx = SlabAllocator::index_from_size(was_allocated.1);

            let mut slab_is_fully_free     = false;
            {
                let mut linked_list_allocator = &mut self.linked_list_allocator;
                let mut frame_allocator = &mut self.frame_allocator;

                if let Some(ref mut slab) = &mut self.size_to_slab[slab_array_idx] {
                    slab.free(pointer, linked_list_allocator, frame_allocator);

                    slab_is_fully_free = slab.is_fully_free();
                }
            }

            if  slab_is_fully_free {
                self.size_to_slab[slab_array_idx] = None;
            }

            //self.address_to_size.delete((was_allocated.leak()));
        }
    }

    fn assigned_memory_size() -> usize {
        unimplemented!()
    }

    fn aux_data_structures_size() -> usize {
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