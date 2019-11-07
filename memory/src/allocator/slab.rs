use stdx_memory::MemoryAllocator;
use stdx_memory::ConstantSizeMemoryAllocator;
use stdx_memory::MemoryAllocatorMeta;
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
use stdx_memory::collections::immutable::double_linked_list::DoubleLinkedListIterator;
use stdx_memory::heap;
use stdx_memory::heap::RC;
use stdx_memory::heap::Box;
use stdx_memory::heap::RCBox;
use stdx_memory::heap::WeakBox;
use allocator::buddy::BuddyAllocator;
use core::alloc::Alloc;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::alloc::AllocErr;
use core::ptr::NonNull;
use core::ops::DerefMut;
use core::ops::Deref;
use self::free_list::FreeListAllocator;
use multiboot::multiboot_header::MultibootHeader;
use display::vga::writer::Writer;

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
type ProperPtr              = RC<ListOfAllocators, free_list::FreeListAllocator>;

#[repr(C)]
struct Slab {
    // free data structures
    non_empty : avl::AVLTree<ProperPtr, free_list::FreeListAllocator>,
    // allocate data structures
    non_full : Option<ProperPtr>
}

impl Slab {
    fn new(allocation_size: usize, frame_allocator: &mut BuddyAllocator, memory_allocator: &mut free_list::FreeListAllocator, tree_allocator: &mut free_list::FreeListAllocator) -> Option<Self> {
        frame_allocator.allocate(allocation_size).map(|slab_start_address| {

            let slab_size = BuddyAllocator::true_allocation_size_for(allocation_size);
            let allocator = free_list::FreeListAllocator::from_size(slab_start_address, slab_size, allocation_size);
            let strs = allocator.start_address();


            let allocator_boxed = DoubleLinkedList::new_rc(allocator, memory_allocator);
            let cv = allocator_boxed.value().start_address();

            let tree = avl::AVLTree::new(RC::clone(&allocator_boxed), tree_allocator);

            let cvv = allocator_boxed.value().start_address();

            Slab {
                non_empty: tree,
                non_full: Some(allocator_boxed),
            }
        })
    }

    pub fn print_what(&self, writer : &mut Writer) -> ()
        {

        self.non_empty.print_what(|n| n.value().start_address(), writer)
    }

    // slab size is passed here to prevent saving it in slab structure, because slab allocator
    // knows what slabs and of what sizes it has
    fn allocate(&mut self, size : usize, slab_size : usize, frame_allocator : &mut BuddyAllocator, memory_allocator : &mut free_list::FreeListAllocator) -> Option<usize> {
        // check if there is any non-full allocators present,
        // if not - create a new one (increase slab size) and allocate from it
        let non_full_allocation_result = self.try_allocate_non_full(size, frame_allocator, memory_allocator);

        // slab is full - try increase its size
        non_full_allocation_result.or_else(|| {

            let true_slab_size                 = BuddyAllocator::true_allocation_size_for(slab_size);
            let frame_allocation_result = frame_allocator.allocate(true_slab_size);

            // if no frames are available - then its out of mem error
            frame_allocation_result.and_then(|new_frame| {
                let mut new_allocator = free_list::FreeListAllocator::from_size(new_frame, true_slab_size, slab_size);

                // this will always succeed
                let result = new_allocator.allocate_size();

                let new_dlist_cell = match self.non_full {
                    Some(ref mut node) => {
                        DoubleLinkedList::add(&mut RC::clone(node),  new_allocator, memory_allocator)},
                    _ =>
                        DoubleLinkedList::new_rc(new_allocator, memory_allocator)
                };

                self.non_empty.insert(RC::clone(&new_dlist_cell), memory_allocator);
                self.non_full = Some(new_dlist_cell);

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

    fn free_from_non_full(&mut self, pointer : usize, memory_allocator : &mut free_list::FreeListAllocator, frame_allocator : &mut BuddyAllocator, printer : &mut Writer) {
        let mut allocator_is_empty = false;
        use core::fmt::Write;
        use core::fmt::Display;

        for e in DoubleLinkedListIterator::new(self.non_full.clone()) {
            let x = e.value().start_address();
            let w = x;

            writeln!(printer, "---- Llist value {}", x);
        }

        // let the allocator perform free
        if let Some(ref mut non_full_dlist) = self.non_full {
            let mut allocator = non_full_dlist.value_mut();

            allocator.free_size(pointer);

            allocator_is_empty = allocator.fully_free();
        }

            for e in DoubleLinkedListIterator::new(self.non_full.clone()) {
            let x = e.value().start_address();
            let w = x;

            writeln!(printer, "---- Llist value {}", x);
        }

        // if allocator is empty (every allocated block was freed) then we can reclaim its memory and
        // remove it from dlist and tree
        if allocator_is_empty {
            if let Some(mut head_cell) = self.non_full.take() {
                let head_start_addr = head_cell.value().start_address();

                {
                    let prev = head_cell.prev();

                    self.non_full = prev;
                }

                // take head_cell out of dlist
                DoubleLinkedList::modify_neighbour_connections(head_cell);

                //self.print_what(printer);

                // take head_cell out of a tree (drops the head_cell)
                self.non_empty.delete_by(head_start_addr, |n| n.value().start_address());

                // reclaim frame
                frame_allocator.free(head_start_addr);
            }
        }
        self.print_what(printer);
    }

    fn free_from_non_empty(&mut self, pointer : usize, memory_allocator : &mut free_list::FreeListAllocator, frame_allocator : &mut BuddyAllocator, printer : &mut Writer) {

        let mut dlist_opt = self.non_empty.find_by(&pointer,
                                                   |node| node.value().start_address(),
                                                   |node, ptr| node.value().is_inside_address_space(*ptr));

        self.print_what(printer);
        let mut allocator_is_empty = false;

        // let the allocator perform free
        if let Some(mut dlist) = dlist_opt.as_mut() {
            let mut allocator = dlist.value_mut();
            allocator.free_size(pointer);

            allocator_is_empty = allocator.fully_free();
        }

        // if allocator is empty (every allocated block was freed) then we can reclaim its memory and
        // remove it from dlist and tree
        if allocator_is_empty {
            if let Some(dlist_cell) = dlist_opt {
                let start_address = dlist_cell.value().start_address();

                // take head_cell out of dlist
                DoubleLinkedList::modify_neighbour_connections(dlist_cell.leak());

                self.print_what(printer);

                // take head_cell out of a tree (drops the head_cell)
                self.non_empty.delete_by(start_address, |n| n.value().start_address());

                // reclaim frame
                frame_allocator.free(start_address);
            }
        }
    }

    fn free(&mut self, pointer : usize, memory_allocator : &mut free_list::FreeListAllocator, frame_allocator : &mut BuddyAllocator, printer : &mut Writer) {
        // maybe the pointer belongs to allocator in the head of non_full dlist
        // if not then search for allocator in the address tree
        if  self.address_belongs_to_non_full(pointer)  {
            self.free_from_non_full(pointer, memory_allocator, frame_allocator, printer);
        }
        else {
            self.free_from_non_empty(pointer, memory_allocator, frame_allocator, printer);
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
    end_address               : usize,
    array_allocator         : bump::BumpAllocator,
    tree_allocator            : free_list::FreeListAllocator,
    linked_list_allocator : free_list::FreeListAllocator,
    frame_allocator        : BuddyAllocator,
    printer : &'static mut  Writer
}

impl SlabAllocator {

    pub fn aux_data_structures_size_for(start_address1 : usize, end_address1 : usize) -> usize {
        let start_address      = Frame::address_align_up(start_address1);
        let end_address        = Frame::address_align_down(end_address1);

        assert!(end_address > start_address, "Cannot create allocator when end address <= start address. Start address : {}, end address {}", start_address, end_address);

        let total_memory      = end_address - start_address + 1;

        assert!(total_memory > MIN_ALLOCATION_SIZE, "Cannot create allocator when total memory size < MIN_ALLOCATION_SIZE (32). Start address : {}, end address {}",
                start_address,
                end_address);

        // compute max memory size for inner allocators to work with
        let total_slab_count                             = SlabAllocator::total_slab_count(total_memory);
        let array_size                                         = Array::<Option<Slab>>::mem_size_for(total_slab_count);
        let (avl_tree_size, linked_list_size)       = SlabAllocator::buddy_free_list_size(total_slab_count, total_memory);
        let avl_tree_cell_size                           = avl::AVLTree::<free_list::FreeListAllocator, free_list::FreeListAllocator>::cell_size();
        let linked_list_cell_size                       = heap::rc_size_for::<DoubleLinkedList<free_list::FreeListAllocator, free_list::FreeListAllocator>>();

        let this_aux_data_structures_size =  array_size + avl_tree_cell_size + linked_list_cell_size;

        this_aux_data_structures_size + BuddyAllocator::aux_data_structures_size_for(start_address + this_aux_data_structures_size, end_address)
    }

    pub fn frame_allocator(&mut self) -> &mut BuddyAllocator {
        &mut self.frame_allocator
    }

    pub fn is_fully_free(&self) ->bool {
        let slabs_are_empty  = self.size_to_slab.iterator().all(|e| e.is_none());
        let tree_is_empty       = self.address_to_size.is_empty();

         slabs_are_empty && tree_is_empty
    }

    pub fn new(start_address1 : usize, end_address1 : usize, printer : &'static mut  Writer) -> Self {
        let start_address      = Frame::address_align_up(start_address1);
        let end_address        = Frame::address_align_down(end_address1);

        assert!(end_address > start_address, "Cannot create allocator when end address <= start address. Start address : {}, end address {}", start_address, end_address);

        let total_memory      = end_address - start_address + 1;

        assert!(total_memory > MIN_ALLOCATION_SIZE, "Cannot create allocator when total memory size < MIN_ALLOCATION_SIZE (32). Start address : {}, end address {}",
                start_address,
                end_address);

        // compute max memory size for inner allocators to work with
        let total_slab_count                             = SlabAllocator::total_slab_count(total_memory);
        let array_size                                         = Array::<Option<Slab>>::mem_size_for(total_slab_count);
        let (avl_tree_size, linked_list_size)       = SlabAllocator::buddy_free_list_size(total_slab_count, total_memory);
        let avl_tree_cell_size                           = avl::AVLTree::<free_list::FreeListAllocator, free_list::FreeListAllocator>::cell_size();
        let linked_list_cell_size                       = heap::rc_size_for::<DoubleLinkedList<free_list::FreeListAllocator, free_list::FreeListAllocator>>();

        // create inner allocators
        let mut array_allocator             = bump::BumpAllocator::from_address(start_address, array_size);
        let mut tree_allocator               = free_list::FreeListAllocator::from_size(array_allocator.end_address() + 1, avl_tree_size, avl_tree_cell_size);
        let mut linked_list_allocator    = free_list::FreeListAllocator::from_size(tree_allocator.end_address() + 1, linked_list_size, linked_list_cell_size);

        // create allocate/free data structures
        let size_to_slab            = Array::<Option<Slab>>::new(total_slab_count, &mut array_allocator);
        let address_to_size     = avl::AVLTree::<(usize, usize), free_list::FreeListAllocator>::new_empty();

        // create frame allocator
        let frame_allocator = BuddyAllocator::new(linked_list_allocator.end_address(), linked_list_allocator.end_address()+10000);

        SlabAllocator {
            size_to_slab,
            address_to_size,
            total_memory,
            start_address,
            end_address,
            array_allocator,
            tree_allocator,
            linked_list_allocator,
            frame_allocator,
            printer
        }
    }

    fn buddy_free_list_size(buddy_levels_count : usize, total_memory : usize) -> (usize, usize) {
        let mut tree_size = 0;
        let mut linked_list_size = 0;

        for block_count in block_count!(total_memory, buddy_levels_count, MIN_ALLOCATION_SIZE) {
            tree_size           += avl::AVLTree::<free_list::FreeListAllocator, free_list::FreeListAllocator>::cell_size();
            linked_list_size += mem::size_of::<DoubleLinkedList<free_list::FreeListAllocator, free_list::FreeListAllocator>>();
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
                let mut printerS = &mut self.printer;

                // check if we have existing slab for requested size,
                // if not - try create a new slab for this size
                let result_from_existing_slab = {
                    let mut slab_opt = &mut self.size_to_slab[size_array_idx];

                    slab_opt.as_mut().and_then(|mut slab| {

                        //slab.print_what(printerS);

                        let result = SlabAllocator::allocate0(
                            size_rounded,
                            slab,
                            frame_allocator,
                            linked_list_allocator,
                            tree_allocator,
                            address_to_size);

                        //slab.print_what(printerS);

                        result
                    })
                };

                // if no slab is found, then try create a new one
                let mut size_to_slab  = &mut self.size_to_slab;
                result_from_existing_slab.or_else(|| {
                    let new_slab_opt = Slab::new(size_rounded, frame_allocator, linked_list_allocator, tree_allocator);

                    // if slab cannot be created - then its oom
                    new_slab_opt.and_then(|mut new_slab| {

                        //new_slab.print_what(printerS);

                        let result = SlabAllocator::allocate0(
                            size_rounded,
                            &mut new_slab,
                            frame_allocator,
                            linked_list_allocator,
                            tree_allocator,
                            address_to_size);

                        //new_slab.print_what(printerS);

                        size_to_slab.update(size_array_idx, Some(new_slab));

                        result
                    })
                })
            }
        }
    }

    fn free(&mut self, pointer: usize) {
        let mut printerS = &mut self.printer;
        if let Some(was_allocated) = self.address_to_size.find_by(&pointer, |w| w.0,  |x, y| x .1== *y) {
            let slab_array_idx = SlabAllocator::index_from_size(was_allocated.1);

            let mut slab_is_fully_free = false;
            {
                let mut linked_list_allocator  = &mut self.linked_list_allocator;
                let mut frame_allocator         = &mut self.frame_allocator;

                let mut a = &mut self.size_to_slab[slab_array_idx] ;
                if let Some(ref mut slab) = a {
                    //slab.print_what(printerS);

                    slab.free(pointer, linked_list_allocator, frame_allocator, printerS);

                    //slab.print_what(printerS);

                    slab_is_fully_free = slab.is_fully_free();

                    self.address_to_size.delete(was_allocated.leak());
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
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        self.allocate(layout.size())
            .map(|a| Ok(NonNull::new_unchecked(a as * mut u8)))
            .unwrap_or(Err(AllocErr))
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.free(*ptr.as_ref() as usize)
    }
}

pub struct SlabHelp {
    pub value : Option<NonNull<SlabAllocator>>
}

unsafe impl GlobalAlloc for SlabHelp {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut  v = self.value.clone().unwrap();
        let mut escape = v.as_mut();

        escape.allocate(layout.size())
            .map(|a| a as * mut u8)
            .unwrap_or(0 as * mut u8)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut  v = self.value.clone().unwrap();
        let mut escape = v.as_mut();

        escape.free(ptr as usize)
    }
}