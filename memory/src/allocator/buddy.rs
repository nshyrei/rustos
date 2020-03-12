use stdx_memory::{MemoryAllocator, ConstantSizeMemoryAllocator, MemoryAllocatorMeta};
use stdx_memory::collections::array::Array;
use stdx_memory::collections::double_linked_list::{BuddyMap, UsizeLinkedMap};
use stdx_memory::collections::frame_bitmap::FrameBitMap;
use allocator::bump;
use frame::{Frame, FRAME_SIZE};
use frame::frame_allocator::FrameAllocator;
use stdx::iterator::IteratorExt;
use allocator::free_list;
use allocator;
use stdx::math;
use stdx::Sequence;
use paging;
use paging::page_table;
use multiboot::multiboot_header::MultibootHeader;

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

macro_rules! total_block_count {
    ($total_memory:expr, $total_buddy_levels:expr, $starting_block_size:expr) => {{
        block_count!($total_memory, $total_buddy_levels, $starting_block_size).sum()
    }}
}

macro_rules! block_count_indexed {
    ($total_memory:expr, $total_buddy_levels:expr, $starting_block_size:expr) => {{
        block_count!($total_memory, $total_buddy_levels, $starting_block_size).index_items()
    }}
}

pub struct BuddyAllocator {
    allocation_sizes     : Array<usize>,    
    buddy_free_lists     : Array<BuddyFreeList>,    
    array_allocator      : bump::BumpAllocator,
    free_list_allocator  : free_list::FreeListAllocator,
    start_address          : usize,
    end_address            : usize,
    memory_start_address: usize,
    page_tables_allocator :  free_list::FreeListAllocator
}

impl BuddyAllocator {

    pub fn debug_allocation_size(&self) -> usize {
        self.allocation_sizes[Frame::number_for_address(0)]
    }

    pub fn total_aux_data_structures_size(start_address1 : usize, end_address1 : usize) -> usize {
        let (start_address, end_address)                           = allocator::align_addresses(start_address1, end_address1);
        let total_memory                                                        = allocator::total_memory(start_address, end_address);
        let (total_frames_count, total_buddy_levels)    = BuddyAllocator::frames_and_buddy_levels(start_address, end_address);
        let (array_sizes, buddy_free_lists_size)                = BuddyAllocator::aux_data_structures_size(total_frames_count, total_buddy_levels, total_memory);

        let this_aux_structures_size = array_sizes + buddy_free_lists_size;

        this_aux_structures_size + free_list::FreeListAllocator::aux_data_structures_size_for(buddy_free_lists_size, BuddyMap::cell_size())
    }

    fn frames_and_buddy_levels(start_address : usize, end_address : usize) -> (usize, usize) {
        assert!(end_address > start_address, "Cannot create allocator when end address <= start address");

        let total_memory = allocator::total_memory(start_address, end_address);

        let total_frames_count = Frame::from_address(total_memory).number();

        assert!(end_address > start_address, "Cannot create allocator when total memory size < FRAME_SIZE (4096)");

        let total_buddy_levels = BuddyAllocator::total_buddy_levels(total_memory);

        (
            total_frames_count,
            total_buddy_levels
        )
    }

    fn frames_and_buddy_levels1(total_memory : usize) -> (usize, usize) {
        let total_frames_count = Frame::from_address(total_memory).number();
        let total_buddy_levels = BuddyAllocator::total_buddy_levels(total_memory);

        (
            total_frames_count,
            total_buddy_levels
        )
    }

    fn aux_data_structures_size(total_frames_count : usize, total_buddy_levels : usize, total_memory : usize) -> (usize, usize) {

        // compute max memory size for inner allocators to work with
        let sizes_array_size                                                   = Array::<usize>::mem_size_for(total_frames_count);
        let buddy_free_list_array_size                              = Array::<BuddyFreeList>::mem_size_for(total_buddy_levels);
        let (buddy_array_size, buddy_free_lists_size) = BuddyAllocator::buddy_free_list_size(total_buddy_levels, total_memory);

        let array_sizes = sizes_array_size + buddy_array_size + buddy_free_list_array_size;

        (array_sizes, buddy_free_lists_size)
    }

    pub fn true_allocation_size_for(size : usize) -> usize {
        if size < FRAME_SIZE {
            FRAME_SIZE
        } else {
            size
        }
    }

    pub fn new2(start_address : usize, total_memory : usize,  end_address : usize) -> Self {
        let (total_frames_count, total_buddy_levels) = BuddyAllocator::frames_and_buddy_levels1(total_memory);

        // compute max memory size for inner allocators to work with
        let (array_sizes, buddy_free_lists_size)                = BuddyAllocator::aux_data_structures_size(total_frames_count, total_buddy_levels, total_memory);

        // create inner allocators
        let mut array_allocator     = bump::BumpAllocator::from_address(start_address, array_sizes);
        let mut free_list_allocator = free_list::FreeListAllocator::from_size(
            array_allocator.end_address() + 1,
            buddy_free_lists_size,
            BuddyMap::cell_size());

        // create allocate/free data structures
        let allocation_sizes                            = Array::<usize>::new(total_frames_count, &mut array_allocator);
        let mut buddy_free_lists_array    = Array::<BuddyFreeList>::new(total_buddy_levels, &mut array_allocator);

        BuddyAllocator::populate_buddy_free_lists(
            &mut buddy_free_lists_array,
            &mut array_allocator,
            &mut free_list_allocator,
            total_buddy_levels,
            total_memory);

        // set initial block that covers all memory as free
        let idx = if total_buddy_levels > 0 { total_buddy_levels - 1} else { 0 };
        buddy_free_lists_array[idx].set_free(0, &mut free_list_allocator);

        // user space memory start
        let memory_start_address = Frame::address_align_up(free_list_allocator.end_address() + 1);

        // create page tables allocator
        let page_tables_allocator = free_list::FreeListAllocator::from_size(free_list_allocator.end_address() + 1, total_memory, FRAME_SIZE);
        let proper_start_address = page_tables_allocator.object_allocation_start_address();

        BuddyAllocator {
            allocation_sizes,
            buddy_free_lists            : buddy_free_lists_array,
            end_address,
            array_allocator,
            free_list_allocator,
            start_address : proper_start_address,
            memory_start_address,
            page_tables_allocator
        }
    }

    pub fn new(start_address1 : usize, end_address1 : usize) -> Self {
        let (start_address, end_address)                           = allocator::align_addresses(start_address1, end_address1);
        let total_memory                                                        = allocator::total_memory(start_address, end_address);
        let (total_frames_count, total_buddy_levels)    = BuddyAllocator::frames_and_buddy_levels(start_address, end_address);

        // compute max memory size for inner allocators to work with
        let (array_sizes, buddy_free_lists_size)                = BuddyAllocator::aux_data_structures_size(total_frames_count, total_buddy_levels, total_memory);

        // create inner allocators
        let mut array_allocator     = bump::BumpAllocator::from_address(start_address, array_sizes);
        let mut free_list_allocator = free_list::FreeListAllocator::from_size(
            array_allocator.end_address() + 1, 
            buddy_free_lists_size,
            BuddyMap::cell_size());

        // create allocate/free data structures
        let allocation_sizes                            = Array::<usize>::new(total_frames_count, &mut array_allocator);
        let mut buddy_free_lists_array    = Array::<BuddyFreeList>::new(total_buddy_levels, &mut array_allocator);

        BuddyAllocator::populate_buddy_free_lists(
            &mut buddy_free_lists_array, 
            &mut array_allocator, 
            &mut free_list_allocator,
            total_buddy_levels,
            total_memory);

        // set initial block that covers all memory as free
        let idx = if total_buddy_levels > 0 { total_buddy_levels - 1} else { 0 };
        buddy_free_lists_array[idx].set_free(0, &mut free_list_allocator);

        // user space memory start
        let memory_start_address = Frame::address_align_up(free_list_allocator.end_address() + 1);

        // create page tables allocator
        let page_tables_allocator = free_list::FreeListAllocator::from_address(free_list_allocator.end_address() + 1, end_address, FRAME_SIZE);
        let proper_start_address = page_tables_allocator.object_allocation_start_address();

        BuddyAllocator {
            allocation_sizes,
            buddy_free_lists            : buddy_free_lists_array,            
            end_address,
            array_allocator,
            free_list_allocator,
            start_address : proper_start_address,
            memory_start_address,
            page_tables_allocator
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

    fn populate_buddy_free_lists(buddy_free_lists : &mut Array<BuddyFreeList>,
        array_allocator : &mut bump::BumpAllocator,
        free_list_allocator : &mut free_list::FreeListAllocator,
        total_buddy_levels : usize,
        total_memory : usize)
    {        
        for (block_count, i) in block_count_indexed!(total_memory, total_buddy_levels, FRAME_SIZE) {
            let buddy_free_list = BuddyFreeList::new(block_count, array_allocator);            
            buddy_free_lists.update(i, buddy_free_list); 
        }
    }

    fn buddy_free_list_size(total_buddy_levels_count : usize, total_memory : usize) -> (usize, usize) {
        let total_block_count = total_block_count!(total_memory, total_buddy_levels_count, FRAME_SIZE);

        let array_size      = BuddyMap::mem_size_for_array(total_block_count) + BuddyFreeList::mem_size_for_array(total_block_count);
        let free_list_size = BuddyMap::mem_size_for_linked_list(total_block_count);

        (
            array_size,
            free_list_size
        )
    }

    fn search_free_list_up(&self, size_from : usize) -> Option<usize> {
        let list_length = self.buddy_free_lists.length();        
        let mut i       = BuddyAllocator::index_from_size(size_from);        

        loop {
            if i > list_length - 1 {
                return None
            }
            else if self.buddy_free_lists[i].has_free_block() {
                return Some(i)
            }
            else {
                i += 1                
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

    fn split_down(&mut self, start_index : usize, allocation_size : usize) -> Option<(isize, usize)> {
        let mut i = start_index as isize;
        let mut current_level_size = BuddyAllocator::block_size_from_index(start_index);

        loop {
            // if size < current_level_size at index 0 the algorithm will crash!
            /*if i < 0 {
                return None
            }*/

            let block_index  = self.buddy_free_lists[i]                                   
                                   .first_free_block(&mut self.free_list_allocator)
                                   .unwrap();

            // we can return current block or split it at that point,
            // both operations will set the block to 'in use'
            self.buddy_free_lists[i].set_in_use(block_index, &mut self.free_list_allocator);

            if allocation_size == current_level_size {
                return Some((i, block_index * current_level_size))
            }
            else if i == 0 {
                return None
            }
            else {
                
                let lower_level_block_index = block_index * 2;
                
                // important to set left before right to in use, for it to appear
                // on top of the free blocks stack. The reason for this is that
                // because of allocator convention, e.g. picking left blocks first.
                
                self.buddy_free_lists[i - 1].set_free(lower_level_block_index, &mut self.free_list_allocator);
                self.buddy_free_lists[i - 1].set_free(lower_level_block_index + 1, &mut self.free_list_allocator);
                
                i -= 1;
                current_level_size /= 2;
            }
        }
    }

    fn merge_up(&mut self, pointer : usize, start_index : usize) {
        let buddy_lists_count    = self.buddy_free_lists.length();

        // index across buddy list array
        let mut buddy_list_index = start_index;
        let mut block_index      = BuddyAllocator::address_to_index(pointer, buddy_list_index);

        loop {

            let buddy_free_list       = &mut self.buddy_free_lists[buddy_list_index];
            let block_is_in_use = buddy_free_list.is_in_use(block_index);

            // if we encountered top block
            if buddy_list_index == buddy_lists_count - 1 && block_is_in_use {
                buddy_free_list.set_free(0, &mut self.free_list_allocator);
                break;
            }
            else if buddy_list_index > buddy_lists_count - 1 {
                break;
            }        
            
            let buddy_index       = BuddyFreeList::buddy_index(block_index);

            let block_not_merging = !buddy_free_list.is_merging(block_index);
            let buddy_not_merging = !buddy_free_list.is_merging(buddy_index);
            
            let buddy_is_free     = buddy_free_list.is_free(buddy_index);

            // if current block can be freed and buddy is also free and both doesn't have a merge status            
            // then we can perfrom a merge
            if block_is_in_use 
                && buddy_is_free
                && block_not_merging
                && buddy_not_merging 
            {
                buddy_free_list.set_merge_status(block_index, true);                

                // current block is already set in use (by allocate), so if we can perform merge
                // there is no point in setting it again
                buddy_free_list.set_as_merging(buddy_index, &mut self.free_list_allocator);                

                // in case we are freeing right block and found its left buddy we must set
                // pointer to left buddy address to have proper index in next level buddy list
                block_index /= 2;
                buddy_list_index += 1;
            }

            // only current block can be freed
            else if block_is_in_use && block_not_merging {
                buddy_free_list.set_free(block_index, &mut self.free_list_allocator);
                break;
            }
            // current block is already freed somehow -> do nothing
            else {
                break;
            }
        };
    }
}


impl MemoryAllocatorMeta for BuddyAllocator {
    fn start_address(&self) -> usize {
        self.start_address
    }

    fn end_address(&self) -> usize {
        self.end_address
    }

    fn aux_data_structures_size(&self) -> usize {
        self.array_allocator.full_size() +
            self.free_list_allocator.full_size() +
            self.page_tables_allocator.aux_data_structures_size()
    }
}

impl MemoryAllocator for BuddyAllocator {

    fn allocate(&mut self, size : usize) -> Option<usize> {

        if size == 0 {
            None
        }
        else {
            let allocation_size_rounded0 = (2 as usize).pow(math::log2_align_up(size) as u32);
            let allocation_size_rounded = if allocation_size_rounded0 < FRAME_SIZE { 
                FRAME_SIZE
            } else {
                allocation_size_rounded0
            };

            if allocation_size_rounded > self.full_size() {
                None
            }
            else {            
                // Search buddy tree for free blocks on current level denoted by 'buddy_list_index',
                // if nothing found search buddy tree upwards for bigger block that can be splitted.
                // Split bigger block (if any) and propagate split results downwards,
                // until block of required size is created.
                let result = self.search_free_list_up(allocation_size_rounded)
                                .and_then(|index| self.split_down(index, allocation_size_rounded));
                
                if let Some((new_buddy_index, result_address)) = result {
                    let frame_number = Frame::number_for_address(result_address);

                    self.allocation_sizes[frame_number] = new_buddy_index as usize;

                    let result_address = result_address + self.memory_start_address;

                    // map page frames
                    let p4_table = paging::p4_table();
                    let frame_count = Frame::number_for_address(allocation_size_rounded);
                    let proper_frame_count =  if frame_count == 0 { 1 } else { frame_count };
                    p4_table.map_pages_1_to_1(result_address, proper_frame_count , page_table::PRESENT | page_table::WRITABLE, &mut self.page_tables_allocator);

                    let debug = self.allocation_sizes[Frame::number_for_address(0)];

                    Some(result_address)
                }
                else {
                    None
                }
            }
        }
    }

    fn free(&mut self, pointer : usize) {
        let normalized_pointer = pointer - self.memory_start_address;
        let frame_number       = Frame::number_for_address(normalized_pointer);
        let buddy_list_index   = self.allocation_sizes[frame_number];

        self.merge_up(normalized_pointer, buddy_list_index);

        // free page frames
        let p4_table = paging::p4_table();
        unsafe { p4_table.unmap_pages(pointer, buddy_list_index); }
    }
}

struct BuddyFreeList {
    buddy_map      : BuddyMap,
    merge_status   : Array<bool>, //change to bitmap to conserve memory    
}

impl BuddyFreeList {

    fn new<A>(length : usize, memory_allocator : &mut A) -> Self 
    where A : MemoryAllocator {
        let map = UsizeLinkedMap::new(length, memory_allocator);
        BuddyFreeList {
            buddy_map    : BuddyMap(map),
            merge_status : Array::new(length, memory_allocator)
        }
    }        
    
    fn set_free<A>(&mut self, block_index : usize, free_list_allocator : &mut A)
    where A : ConstantSizeMemoryAllocator
    {
        self.buddy_map.add_if_no_key(block_index, free_list_allocator);
        self.merge_status[block_index] = false;
    }

    fn set_in_use<A>(&mut self, block_index : usize, free_list_allocator : &mut A)
    where A : ConstantSizeMemoryAllocator 
    {
        self.buddy_map.0.remove(block_index, free_list_allocator);   
        self.merge_status[block_index] = false;        
    }

    fn first_free_block<A>(&mut self, free_list_allocator : &mut A) -> Option<usize>
    where A : ConstantSizeMemoryAllocator 
    {
        self.buddy_map.first_free_block(free_list_allocator)
    }

    fn set_as_merging<A>(&mut self, block_index : usize, free_list_allocator : &mut A)
    where A : ConstantSizeMemoryAllocator 
    {
        self.set_merge_status(block_index, true);
        self.set_in_use(block_index, free_list_allocator);        
    }

    /// Determines if block is free to use
    /// # Arguments
    /// * `block_start_address` - start address of memory block
    fn is_free(&self, index : usize) -> bool {
        self.buddy_map.0.has_key(index)
    }

    /// Determines if block is occupied
    /// # Arguments
    /// * `block_start_address` - start address of memory
    fn is_in_use(&self, index : usize) -> bool {
        !self.is_free(index)
    }
    
    fn has_free_block(&self) -> bool {
        self.buddy_map.0.has_value()
    }    

    fn is_merging(&self, block_index : usize) -> bool {
        self.merge_status[block_index]
    }

    fn set_merge_status(&mut self, block_index : usize, new_status : bool) {
        self.merge_status[block_index] = new_status;
    }

    fn mem_size_for_array(length : usize) -> usize {        
        Array::<bool>::mem_size_for(length)
    }

    fn buddy_index(i : usize) -> usize {        
        if math::is_even(i) {
            i + 1
        }
        else {
            i - 1
        }
    }    
}