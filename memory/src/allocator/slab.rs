use stdx_memory::{MemoryAllocator, ConstantSizeMemoryAllocator, MemoryAllocatorMeta};
use stdx_memory::collections::array::Array;
use stdx_memory::collections::immutable::double_linked_list::DoubleLinkedList;
use stdx_memory::collections::linked_list::LinkedList;
use stdx_memory::trees::avl;
use allocator::bump;
use allocator::free_list::FreeListAllocator;
use allocator::buddy::BuddyAllocator;
use allocator;
use stdx::iterator::IteratorExt;
use stdx::{Iterable,Sequence} ;
use stdx::math;
use stdx_memory::heap;
use core::cmp;
use core::mem;
use core::alloc::Alloc;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::alloc::AllocErr;
use core::ptr;
use display::vga::writer::Writer;
use frame::FRAME_SIZE;
use core::ops::DerefMut;
use core::ops::Deref;

const MIN_ALLOCATION_SIZE : usize = 32; //bytes

type ListOfAllocators = DoubleLinkedList<SlabCell, bump::ConstSizeBumpAllocator>;
type ProperPtr              = heap::RC<ListOfAllocators, bump::ConstSizeBumpAllocator>;

#[repr(C)]
struct Slab {
    // free data structures
    non_empty : avl::AVLTree<ProperPtr, bump::ConstSizeBumpAllocator>,
    // allocate data structures
    non_full : Option<ProperPtr>
}

#[repr(C)]
struct SlabCell {

    value : FreeListAllocator,

    tree_cell_allocator : bump::ConstSizeBumpAllocator,

    dlist_cell_allocator : bump::ConstSizeBumpAllocator,
}

impl SlabCell {
    fn new(frame_start_address : usize) -> Self {
        let avl_node_size = SlabAllocator::avl_tree_cell_size();
        let dlist_node_size = SlabAllocator::linked_list_cell_size();
        let slab_cell_size = heap::rc_size_for::<SlabCell>();

        let slab_size = BuddyAllocator::true_allocation_size_for(allocation_size) - avl_node_size - dlist_node_size - slab_cell_size;

        let allocator = FreeListAllocator::from_size(slab_start_address, slab_size, allocation_size);

        let tree_cell_allocator = bump::ConstSizeBumpAllocator::from_size(
            allocator.end_address() + 1,
            avl_node_size,
            avl_node_size);

        let dlist_cell_allocator = bump::ConstSizeBumpAllocator::from_size(
            tree_cell_allocator.end_address() + 1,
            dlist_node_size,
            dlist_node_size);

        SlabCell {
            value : allocator,
            tree_cell_allocator,
            dlist_cell_allocator
        }
    }
}

impl cmp::Ord for SlabCell {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl cmp::PartialOrd for SlabCell {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl cmp::Eq for SlabCell {

}

impl cmp::PartialEq for SlabCell {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl Slab {
    fn new(allocation_size: usize, frame_allocator: &mut BuddyAllocator) -> Option<Self> {
        frame_allocator.allocate(allocation_size).map(|slab_start_address| {

            let mut slab_cell = SlabCell::new(slab_start_address);

            let slab_cell_boxed = DoubleLinkedList::new_rc(slab_cell, &mut slab_cell.dlist_cell_allocator);

            let tree = avl::AVLTree::new(heap::RC::clone(&slab_cell_boxed), &mut slab_cell.tree_cell_allocator);

            Slab {
                non_empty: tree,
                non_full: Some(slab_cell_boxed),
            }
        })
    }

    // slab size is passed here to prevent saving it in slab structure, because slab allocator
    // knows what slabs and of what sizes it has
    fn allocate(&mut self, size : usize, slab_size : usize, frame_allocator : &mut BuddyAllocator) -> Option<usize> {
        // check if there is any non-full allocators present,
        // if not - create a new one (increase slab size) and allocate from it
        let non_full_allocation_result = self.try_allocate_non_full();
        // slab is full - try increase its size
        non_full_allocation_result.or_else(|| {

            let true_slab_size                 = BuddyAllocator::true_allocation_size_for(slab_size);
            let frame_allocation_result = frame_allocator.allocate(true_slab_size);

            // if no frames are available - then its out of mem error
            frame_allocation_result.and_then(|new_frame| {

                let mut new_slab_cell = SlabCell::new(new_frame);

                // this will always succeed
                let result = new_slab_cell.value.allocate_size();

                let new_dlist_cell = match self.non_full {
                    Some(ref mut node) => {
                        DoubleLinkedList::add(&mut heap::RC::clone(node),  new_slab_cell, &mut new_slab_cell.dlist_cell_allocator)},
                    _ =>
                        DoubleLinkedList::new_rc(new_slab_cell, &mut new_slab_cell.dlist_cell_allocator)
                };

                self.non_empty.insert(heap::RC::clone(&new_dlist_cell), &mut new_slab_cell.tree_cell_allocator);
                self.non_full = Some(new_dlist_cell);

                result
            })
        })
    }

    fn try_allocate_non_full(&mut self) -> Option<usize> {
        self.non_full
            .as_mut()
            .and_then(|allocator_list| allocator_list.value_mut().value.allocate_size())
    }

    fn address_belongs_to_non_full(&self, pointer : usize) -> bool {
        self.non_full
                .as_ref()
                .map(|list_cell| list_cell.value().value.is_inside_address_space(pointer) )
                .unwrap_or(false)
    }

    fn free_from_non_full(&mut self, pointer : usize, frame_allocator : &mut BuddyAllocator) {
        let mut allocator_is_empty = false;

        // let the allocator perform free
        if let Some(ref mut non_full_dlist) = self.non_full {
            let allocator = &mut non_full_dlist.value_mut().value;

            allocator.free_size(pointer);

            allocator_is_empty = allocator.fully_free();
        }

        // if allocator is empty (every allocated block was freed) then we can reclaim its memory and
        // remove it from dlist and tree
        if allocator_is_empty {
            if let Some(head_cell) = self.non_full.take() {
                let head_start_addr = head_cell.value().start_address();

                {
                    let prev = head_cell.prev();

                    self.non_full = prev;
                }

                // take head_cell out of dlist
                DoubleLinkedList::modify_neighbour_connections(head_cell);

                // take head_cell out of a tree (drops the head_cell)
                self.non_empty.delete_by(head_start_addr, |n| n.value().value.start_address());

                // reclaim frame
                frame_allocator.free(head_start_addr);
            }
        }
    }

    fn free_from_non_empty(&mut self, pointer : usize, frame_allocator : &mut BuddyAllocator) {

        let mut dlist_opt = self.non_empty.find_by(&pointer,
                                                   |node| node.value().value.start_address(),
                                                   |node, ptr| node.value().value.is_inside_address_space(*ptr));

        let mut allocator_is_empty = false;

        // let the allocator perform free
        if let Some(dlist) = dlist_opt.as_mut() {
            let allocator = &mut dlist.value_mut().value;
            allocator.free_size(pointer);

            allocator_is_empty = allocator.fully_free();
        }

        // if allocator is empty (every allocated block was freed) then we can reclaim its memory and
        // remove it from dlist and tree
        if allocator_is_empty {
            if let Some(dlist_cell) = dlist_opt {
                let start_address = dlist_cell.value().value.start_address();

                // take head_cell out of dlist
                DoubleLinkedList::modify_neighbour_connections(dlist_cell.leak());

                // take head_cell out of a tree (drops the head_cell)
                self.non_empty.delete_by(start_address, |n| n.value().value.start_address());

                // reclaim frame
                frame_allocator.free(start_address);
            }
        }
    }

    fn free(&mut self, pointer : usize, frame_allocator : &mut BuddyAllocator) {
        // maybe the pointer belongs to allocator in the head of non_full dlist
        // if not then search for allocator in the address tree
        if  self.address_belongs_to_non_full(pointer)  {
            self.free_from_non_full(pointer, frame_allocator);
        }
        else {
            self.free_from_non_empty(pointer, frame_allocator);
        }
    }

    fn is_fully_free(&self) -> bool {
        self.non_full.is_none() && self.non_empty.is_empty()
    }
}

pub struct SlabAllocator {
    size_to_slab                : Array<Option<Slab>>,
    //address_to_size         : avl::AVLTree<(usize, usize), FreeListAllocator>,
    start_address             : usize,
    end_address               : usize,
    array_allocator         : bump::BumpAllocator,
    tree_allocator            : FreeListAllocator,
    linked_list_allocator :  FreeListAllocator,
    frame_allocator        : BuddyAllocator,
}

type DlistOfAllocators = DoubleLinkedList<FreeListAllocator, FreeListAllocator>;

impl SlabAllocator {

    pub fn total_aux_data_structures_size(start_address1 : usize, end_address1 : usize) -> usize {
        let (start_address, end_address)                        = allocator::align_addresses(start_address1, end_address1);
        let total_slab_count          = SlabAllocator::slab_count(start_address, end_address);

        let (array_size, avl_tree_size, linked_list_size)   = SlabAllocator::aux_data_structures_size(total_slab_count);

        let this_aux_data_structures_size =  array_size + avl_tree_size + linked_list_size;

        this_aux_data_structures_size + BuddyAllocator::total_aux_data_structures_size(start_address + this_aux_data_structures_size + 1, end_address)
    }

    fn avl_tree_cell_size() -> usize {
        avl::AVLTree::<FreeListAllocator, FreeListAllocator>::cell_size()
    }

    fn linked_list_cell_size() -> usize {
        heap::rc_size_for::<DlistOfAllocators>()
    }

    fn slab_count(start_address : usize, end_address : usize) -> usize {
        assert!(end_address > start_address, "Cannot create allocator when end address <= start address. Start address : {}, end address {}", start_address, end_address);

        let total_memory = allocator::total_memory(start_address, end_address);

        assert!(total_memory > MIN_ALLOCATION_SIZE, "Cannot create allocator when total memory size < MIN_ALLOCATION_SIZE (32). Start address : {}, end address {}",
                start_address,
                end_address);

        SlabAllocator::total_slab_count(total_memory)
    }

    fn aux_data_structures_size(total_slab_count : usize) -> (usize, usize, usize) {

        let array_size             = Array::<Option<Slab>>::mem_size_for(total_slab_count);
        let avl_tree_size        = SlabAllocator::avl_tree_cell_size() * total_slab_count;
        let linked_list_size    = SlabAllocator::linked_list_cell_size() * (31457281 / 4096);

        (
            array_size,
            avl_tree_size,
            linked_list_size
        )
    }

    pub fn frame_allocator(&mut self) -> &mut BuddyAllocator {
        &mut self.frame_allocator
    }

    pub fn is_fully_free(&self) ->bool {
        let slabs_are_empty  = self.size_to_slab.iterator().all(|e| e.is_none());
        //let tree_is_empty       = self.address_to_size.is_empty();

         slabs_are_empty// && tree_is_empty
    }

    pub fn new2(start_address : usize,  total_memory : usize, end_address : usize, printer : &'static mut  Writer) -> Self {

        let mut frame_allocator = BuddyAllocator::new2(start_address, total_memory, end_address);

        let total_slab_count = SlabAllocator::total_slab_count(total_memory);

        let (array_size, avl_tree_size, linked_list_size)   = SlabAllocator::aux_data_structures_size(total_slab_count);

        // create inner allocators
        let mut array_allocator             = bump::BumpAllocator::from_address(start_address, array_size);
        let mut tree_allocator               = FreeListAllocator::from_size(array_allocator.end_address() + 1, avl_tree_size, SlabAllocator::avl_tree_cell_size());
        let mut linked_list_allocator    = FreeListAllocator::from_size(tree_allocator.end_address() + 1, linked_list_size, SlabAllocator::linked_list_cell_size());

        // create allocate/free data structures
        let size_to_slab            = Array::<Option<Slab>>::new(total_slab_count, &mut frame_allocator);
        //let address_to_size     = avl::AVLTree::<(usize, usize), FreeListAllocator>::new_empty();

        // create frame allocator
        //let frame_allocator = BuddyAllocator::new2(linked_list_allocator.end_address() + 1, total_memory, end_address);

        SlabAllocator {
            size_to_slab,
            //address_to_size,
            start_address,
            end_address,
            array_allocator,
            tree_allocator,
            linked_list_allocator,
            frame_allocator
        }
    }

    pub fn new(start_address1 : usize, end_address1 : usize, printer : &'static mut  Writer) -> Self {
        // compute max memory size for inner allocators to work with

        let (start_address, end_address)                        = allocator::align_addresses(start_address1, end_address1);
        let total_slab_count                                                = SlabAllocator::slab_count(start_address, end_address);

        let (array_size, avl_tree_size, linked_list_size)   = SlabAllocator::aux_data_structures_size(total_slab_count);

        // create inner allocators
        let mut array_allocator             = bump::BumpAllocator::from_address(start_address, array_size);
        let tree_allocator               = FreeListAllocator::from_size(array_allocator.end_address() + 1, avl_tree_size, SlabAllocator::avl_tree_cell_size());
        let linked_list_allocator    = FreeListAllocator::from_size(tree_allocator.end_address() + 1, linked_list_size, SlabAllocator::linked_list_cell_size());

        // create allocate/free data structures
        let size_to_slab            = Array::<Option<Slab>>::new(total_slab_count, &mut array_allocator);
        let address_to_size     = avl::AVLTree::<(usize, usize), FreeListAllocator>::new_empty();

        // create frame allocator
        let frame_allocator = BuddyAllocator::new(linked_list_allocator.end_address() + 1, end_address);

        SlabAllocator {
            size_to_slab,
            //address_to_size,
            start_address,
            end_address,
            array_allocator,
            tree_allocator,
            linked_list_allocator,
            frame_allocator
        }
    }

    fn free_with_size_hint(&mut self, pointer: usize, size: usize) {
        if size >= FRAME_SIZE {
            self.frame_allocator.free(pointer)
        } else {
            let slab_array_idx = SlabAllocator::index_from_size(size);

            let mut slab_is_fully_free = false;
            {
                let linked_list_allocator = &mut self.linked_list_allocator;
                let frame_allocator = &mut self.frame_allocator;

                if let Some(ref mut slab) = &mut self.size_to_slab[slab_array_idx] {
                    slab.free(pointer, frame_allocator);

                    slab_is_fully_free = slab.is_fully_free();
                }
            }

            if slab_is_fully_free {
                self.size_to_slab[slab_array_idx] = None;
            }
        }
    }

    fn buddy_free_list_size(buddy_levels_count : usize, total_memory : usize, tree_cell_size : usize, linked_list_cell_size : usize) -> (usize, usize) {
        let mut tree_size = 0;
        let mut linked_list_size = 0;

        for block_count in block_count!(total_memory, buddy_levels_count, MIN_ALLOCATION_SIZE) {
            tree_size           += tree_cell_size * block_count;
            linked_list_size += linked_list_cell_size * block_count;
        }

        (tree_size, linked_list_size)
    }

    fn total_slab_count(total_memory : usize) -> usize {
        let idx = SlabAllocator::index_from_size(total_memory);

        if idx > 0 {
            idx + 1
        } else {
            1
        }
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
        let allocation_size_rounded0 = (2 as usize).pow(math::log(2, size) as u32);

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
        address_to_size : &mut avl::AVLTree<(usize, usize), FreeListAllocator>) -> Option<usize> {

        slab.allocate(
            size_rounded,
            size_rounded,
            frame_allocator
        ).map(|r| {
            //address_to_size.insert((r, size_rounded), tree_allocator);
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

            if size_rounded > self.full_size() {
                None
            }else if size_rounded >= FRAME_SIZE {
                self.frame_allocator.allocate(size_rounded)
            }
                else {
                let size_array_idx = SlabAllocator::index_from_size(size_rounded);

                let  frame_allocator            = &mut self.frame_allocator;
                let  linked_list_allocator     = &mut self.linked_list_allocator;
                let  tree_allocator               = &mut self.tree_allocator;
//                let  address_to_size            = &mut self.address_to_size;

                // check if we have existing slab for requested size,
                // if not - try create a new slab for this size
                let result_from_existing_slab = {
                    let slab_opt = &mut self.size_to_slab[size_array_idx];

                    slab_opt.as_mut().and_then(|slab| {

                        let result = SlabAllocator::allocate0(
                            size_rounded,
                            slab,
                            frame_allocator,
                            linked_list_allocator,
                            tree_allocator,
                            address_to_size);

                        result
                    })
                };

                // if no slab is found, then try create a new one
                let size_to_slab  = &mut self.size_to_slab;
                result_from_existing_slab.or_else(|| {
                    let new_slab_opt = Slab::new(size_rounded, frame_allocator);

                    // if slab cannot be created - then its oom
                    new_slab_opt.and_then(|mut new_slab| {

                        let result = SlabAllocator::allocate0(
                            size_rounded,
                            &mut new_slab,
                            frame_allocator,
                            linked_list_allocator,
                            tree_allocator,
                            address_to_size);

                        size_to_slab.update(size_array_idx, Some(new_slab));

                        result
                    })
                })
            }
        }
    }

    fn free(&mut self, pointer: usize) {
        /*if let Some(was_allocated) = self.address_to_size.find_by(&pointer, |w| w.0,  |x, y| x .1== *y)*/ {
            let slab_array_idx = SlabAllocator::index_from_size(was_allocated.1);

            let mut slab_is_fully_free = false;
            {
                let linked_list_allocator  = &mut self.linked_list_allocator;
                let frame_allocator         = &mut self.frame_allocator;

                if let Some(ref mut slab) = &mut self.size_to_slab[slab_array_idx]  {

                    slab.free(pointer, frame_allocator);

                    slab_is_fully_free = slab.is_fully_free();

                    //self.address_to_size.delete(was_allocated.leak());
                }
            }

            if  slab_is_fully_free {
                self.size_to_slab[slab_array_idx] = None;
            }
        }
    }
}

impl MemoryAllocatorMeta for SlabAllocator {
    fn start_address(&self) -> usize {
        self.start_address
    }

    fn end_address(&self) -> usize {
        self.end_address
    }

    fn aux_data_structures_size(&self) -> usize {
        self.array_allocator.full_size() +
            self.tree_allocator.full_size() +
            self.linked_list_allocator.full_size() +
            self.frame_allocator.aux_data_structures_size()
    }
}

unsafe impl Alloc for SlabAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<ptr::NonNull<u8>, AllocErr> {
        self.allocate(layout.size())
            .map(|a| Ok(ptr::NonNull::new_unchecked(a as * mut u8)))
            .unwrap_or(Err(AllocErr))
    }

    unsafe fn dealloc(&mut self, ptr : ptr::NonNull<u8>, layout: Layout) {
        self.free(*ptr.as_ref() as usize)
    }
}

pub struct SlabHelp {
    pub value : ptr::NonNull<SlabAllocator>
}

impl SlabHelp {

    pub fn is_fully_free(&self) -> bool {
        unsafe { self.value.as_ref().is_fully_free() }
    }
}

unsafe impl GlobalAlloc for SlabHelp {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // escape immutable self
        let mut  v = self.value.clone();
        let mut escape = v.as_mut();

        escape.allocate(layout.size())
            .map(|a| a as * mut u8)
            .unwrap_or(0 as * mut u8)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // escape immutable self
        let mut  v = self.value.clone();
        let mut escape = v.as_mut();

        escape.free_with_size_hint(ptr as usize, layout.size())
    }
}