use core::marker;
use core::ops;
use core::fmt;
use frame::Frame;
use frame::FRAME_SIZE;
use frame::frame_allocator::FrameAllocator;
use hardware::x86_64::tlb;
use stdx_memory::MemoryAllocator;

pub const PAGE_TABLE_SIZE : usize = 4096; //4kb, x86-64 spec

pub const PAGE_TABLE_ENTRY_SIZE : usize = 8;

pub type VirtualFrame = Frame;

pub type PhysicalFrame = Frame;

pub type P4Table = PageTable<P4>;

pub trait TableLevel {
    
    fn index_shift() -> usize;

    /// Determines index inside page table based on virtual page.
    /// Uses 'index_shift' to properly extract index based on table level.    
    /// 
    fn page_index(page : VirtualFrame) -> usize {
        (page.number() >> Self::index_shift()) & 511
    }
}

pub trait HasNextTableLevel : TableLevel {
    type NextTableLevel : TableLevel;    
}

pub enum P4 {}
pub enum P3 {}
pub enum P2 {}
pub enum P1 {}

impl TableLevel for P4 {
    fn index_shift() -> usize {
        27
    }
}

impl TableLevel for P3 {
    fn index_shift() -> usize {
        18
    }
}

impl TableLevel for P2 {
    fn index_shift() -> usize {
        9
    }
}

impl TableLevel for P1 {
    fn index_shift() -> usize {
        0
    }
}

impl HasNextTableLevel for P4 {
    type NextTableLevel = P3;
}

impl HasNextTableLevel for P3 {
    type NextTableLevel = P2;
}

impl HasNextTableLevel for P2 {
    type NextTableLevel = P1;
}

#[repr(C)]
pub struct PageTable<Level> where Level : TableLevel
{    
    entries : [PageTableEntry; 512],            // 512 * 8 (sizeof(PageTableEntry)) = 4096 b = 4kb = 1 Frame size
                                                                    // why this size? Because x86-64 spec.
    level : marker::PhantomData<Level>
}

impl<L> ops::Index<usize> for PageTable<L> where L : TableLevel {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &PageTableEntry {        
        &self.entries[index]
    }
}

impl<L> ops::IndexMut<usize> for PageTable<L> where L : TableLevel {
    fn index_mut(&mut self, index : usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

impl <Level> PageTable<Level> where Level : TableLevel {
    pub fn clear_all_entries(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        };
    }
}

// comment out reason: too slow for some reason
//impl <Level> fmt::Display for PageTable<Level> where Level : TableLevel {
    //fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // writeln! has result so its not possible to write 
        // writeln! in for loop        
      //  (0..512).fold(write!(f, "---Page table---"), |_base, i| write!(f, "entry: {}, {} ", i, self[i]))        
    //}    
//}

impl<Level> PageTable<Level> where  Level : HasNextTableLevel {

    pub fn has_next_table(&self, index : usize) -> bool {        
        let table_entry = &self[index];
        let flags = table_entry.flags();

        table_entry.flags().contains(PRESENT) && flags.contains(PRESENT)
    }    

    pub fn next_table_opt(&self, page : VirtualFrame) -> Option<&'static mut PageTable<Level::NextTableLevel>> {
        let index = Level::page_index(page);
        if self.has_next_table(index) {
            Some(self.next_table(index))
        }
        else {
            None
        }
    }

    pub fn next_table_or_create<M>(&mut self, page : VirtualFrame, frame_allocator : &mut M) -> &'static mut PageTable<Level::NextTableLevel> where M : MemoryAllocator {
        // page number is destructured to check if its index points to 
        // valid (present) page table entry. Recursive looping in P4 table is
        // used to physically address the desired table/frame. 
        let index = Level::page_index(page);

        if self.has_next_table(index) {
            self.next_table(index)
        }
        else {
            // create next level table
            let new_table_frame = frame_allocator.allocate(FRAME_SIZE).expect("No memory for page table");

            // set new entry in current table
            self[index].set_frame(Frame::from_address(new_table_frame), PRESENT | WRITABLE);
            
            // clear next level table
            let result = self.next_table(index);
            result.clear_all_entries();

            result            
        }
    }

    fn next_table(&self, index : usize) -> &'static mut PageTable<Level::NextTableLevel> {
        let table_address = self as *const _ as usize;
        let addr = (table_address << 9) | (index << 12);

        unsafe { &mut (*(addr as *mut PageTable<Level::NextTableLevel>)) }  
    }
}

impl PageTable<P4> {

    /// Returns overrall number of mapped memory in bytes  
    // todo: rewrite  
    pub fn total_mapped_memory(&self) -> usize {
        let mut result : usize = 0;

        for entry in self.entries.iter().take(511).filter(|e| e.is_set()) {
            let p4Page = Frame::from_address(entry.address());

            match self.next_table_opt(p4Page) {
                Some(p3) => {
                    for p3Entry in p3.entries.iter().filter(|e| e.is_set()){
                        let p3Page = Frame::from_address(p3Entry.address());

                        match p3.next_table_opt(p3Page) {
                            Some(p2) => {
                                for p2Entry in p2.entries.iter().filter(|e| e.is_set()){
                                    let p1Page = Frame::from_address(p3Entry.address());

                                    match p2.next_table_opt(p1Page) {
                                        Some(p1) =>{
                                            result += p1.entries.iter().filter(|e| e.is_set()).count();
                                        },
                                        None => (),
                                    }
                                }
                            },
                            None => (),
                        }
                    }
                },
                None => (), 
            }
        };

        result * FRAME_SIZE
    }

    /// maps virtual page to physical frame
    ///
    /// # Arguments
    /// * `page` - virtual frame
    /// * `frame` - physical frame
    /// * `frame_allocator` - frame allocator
    pub fn map_page<M>(&mut self, page : VirtualFrame, frame : PhysicalFrame, flags : EntryFlags, frame_allocator : &mut M)  where M : MemoryAllocator {
        let p1 = self.next_table_or_create(page, frame_allocator)
                         .next_table_or_create(page, frame_allocator)
                         .next_table_or_create(page, frame_allocator);

        let p1_index = P1::page_index(page);
        p1[p1_index].set_frame(frame, flags)
    }

    pub fn map<M>(&mut self, virtual_address : usize, physical_address : usize, flags : EntryFlags, frame_allocator : &mut M) where M : MemoryAllocator {
        self.map_page(Frame::from_address(virtual_address), Frame::from_address(physical_address), flags, frame_allocator)
    }

    /// Maps virtual page to physical frame in 1 to 1 fashion, e.g.
    /// virtual page will correspond to physical frame with the same address
    ///
    /// # Arguments
    /// * `page` - virtual frame
    /// * `frame_allocator` - frame allocator
    pub fn map_page_1_to_1<M>(&mut self, page : VirtualFrame, flags : EntryFlags, frame_allocator : &mut M)  where M : MemoryAllocator {
        let frame = page.clone();
        self.map_page(page, frame, flags, frame_allocator);
    }

    pub fn map_1_to_1<M>(&mut self, virtual_address : usize, flags : EntryFlags, frame_allocator : &mut M) where M : MemoryAllocator {
        self.map_page_1_to_1(Frame::from_address(virtual_address), flags, frame_allocator)
    }

    pub fn map_pages_1_to_1<M>(&mut self, virtual_address_start : usize, count : usize, flags : EntryFlags, frame_allocator : &mut M) where M : MemoryAllocator {
        let mut virtual_address = virtual_address_start;

        for _ in 0..count {
            self.map_1_to_1(virtual_address, flags, frame_allocator);
            virtual_address += FRAME_SIZE;
        }
    }

    /// Unmaps virtual page
    ///
    /// # Arguments
    /// * `page` - virtual frame
    /// * `frame_allocator` - frame allocator
    pub unsafe fn unmap_page(&self, page : VirtualFrame) {
        let p1_option = self.next_table_opt(page)
                            .and_then(|p3| p3.next_table_opt(page))
                            .and_then(|p2| p2.next_table_opt(page));

        if let Some(p1) = p1_option {
            let p1_index = P1::page_index(page);
            p1[p1_index].set_unused();

            /*
                Important to flush TLB after unmapping entry to prevent reads from it!!
                Example situation of what will happend if we don't do that: 
                let page = ... 
                map(page, ..., ...)
                let result = translate(page)
                unmap(page) // after this line any direct pointer reads using page.address()
                            //  should produce segfault because we unmapped the page, but they won't
                        //   if we don't flush TLB 
                let should_be_page_fault = *(page.address() as *const u64) // won't produce segfault
            */
            tlb::flush(page.address());
        }    
    }

    pub unsafe fn unmap(&self, virtual_address : usize) {
        self.unmap_page(Frame::from_address(virtual_address))
    }

    pub unsafe fn unmap_pages(&self, virtual_address_start : usize, count : usize) {
        let mut virtual_address = virtual_address_start;

        for _ in 0..count {
            self.unmap(virtual_address);
            virtual_address += FRAME_SIZE;
        }
    }

    /// Translates virtual page to physical frame.
    ///
    /// # Arguments
    /// * `page` - virtual frame
    /// * `frame_allocator` - frame allocator
    ///
    /// # Returns
    /// Some() with physical frame if entry is present for corresponding virtual frame,
    /// otherwise returns None.
    pub fn translate_page(&self, page : VirtualFrame) -> Option<Frame> {
        
        self.next_table_opt(page)
        .and_then(|p3| p3.next_table_opt(page))
        .and_then(|p2| p2.next_table_opt(page))
        .and_then(|p1| { 
            let p1_index = P1::page_index(page);
            let p1_entry = &p1[p1_index];

            if p1_entry.flags().contains(PRESENT) {            
                Some(Frame::from_address(p1_entry.address()))
            }
            else {
                None
            }        
        })
    }

    /// Checks whether virtual page points to existing physical frame
    ///
    /// # Arguments
    /// * `page` - virtual frame
    /// * `frame_allocator` - frame allocator
    ///
    /// # Returns
    /// True if entry is present for corresponding virtual frame, otherwise returns false.
    pub fn is_present(&self, page : VirtualFrame) -> bool {
        self.translate_page(page).is_some()
    }

    /// Translates virtual address to physical address.
    ///
    /// # Arguments
    /// * `page` - virtual address
    /// * `frame_allocator` - frame allocator
    ///
    /// # Returns
    /// Some() with physical address if entry is present for corresponding virtual address,
    /// otherwise returns None.
    pub fn translate(&self, virtual_address : usize) -> Option<usize> {    
        self.translate_page(Frame::from_address(virtual_address)).map(|frame| frame.address() + virtual_address % FRAME_SIZE)
    }

    pub fn set_recursive_entry(&mut self, frame : Frame, flags : EntryFlags) {
        self[511].set_frame(frame, flags);
    }

    /// Performs action on another p4 table through this p4 table.
    /// # Arguments
    /// * `current_p4_table` - current p4 table
    /// * `other_p4_table_address` - frame that holds another p4 table
    /// * `frame_allocator` - frame allocator
    /// * `action` - function to be executed on another p4 table
    /// # Why unsafe
    ///  Uses tlb::flush() which is unsafe
    pub unsafe fn modify_other_table<F, M>(&mut self, other_p4_table_address : Frame, frame_allocator : &mut M, action : F)
    where M : MemoryAllocator,
                F : FnOnce(&mut P4Table, &mut M)
    {
        let current_p4_table = self;
        // 1# map some unused virtual address to point to current p4
        // 2# map some unused virtual address to point to temp p4
        // 3# set recursive entry in temp p4
        // 4# unmap temp p4
        // 5# set recursive entry in current p4 to point to temp p4, this will
        //    make magical address '0xfffffffffffff000' point to temp table (thus not breaking any logic associated with that address)
        // 6# perform modifications on temp4
        // 7# read current p4 through temp virtual address defined in #1
        // 8# restore recursive entry in current p4
        // 9# unmap temp virtual address    

        // map some unused virtual address to point to current p4
        // this will be used to restore recursive mapping in current p4
        // after all the operations with temp p4 
        let p4_physical_address = Frame::from_address(current_p4_table[511].address());   // p4's 511 entry points to self
        let current_p4_save_address = Frame::from_address(0x400000000000);    // some temp address to save current p4
        current_p4_table.map_page(current_p4_save_address, p4_physical_address, PRESENT | WRITABLE, frame_allocator);
        
        // map temp table
        let temp_p4_virtual_address = Frame::from_address(0x200000000000);   // some temp address to map temp p4
        current_p4_table.map_page(temp_p4_virtual_address, other_p4_table_address, PRESENT, frame_allocator);
        
        // set recursive entry in temp table
        let temp_p4 = &mut (*(0x200000000000 as *mut P4Table));
        temp_p4.clear_all_entries();
        temp_p4.set_recursive_entry(other_p4_table_address, PRESENT | WRITABLE);
        
        current_p4_table.unmap_page(temp_p4_virtual_address);

        // set recursive entry of the current p4 to point to temp table
        current_p4_table.set_recursive_entry(other_p4_table_address, PRESENT | WRITABLE);
        
        tlb::flush_all();

        action(current_p4_table, frame_allocator); // reading recursive entry again will move us to the temp table
        
        // read old p4 and place recursive entry back
        let saved_p4 = &mut (*(current_p4_save_address.address() as *mut P4Table));
        saved_p4.set_recursive_entry(p4_physical_address, PRESENT | WRITABLE);
        
        // unmap recursive address saving
        current_p4_table.unmap_page(current_p4_save_address);

        tlb::flush_all();
    }    
}

#[repr(C)]
pub struct PageTableEntry {
    value : u64
}

impl PageTableEntry {

    pub fn new() -> PageTableEntry {
        PageTableEntry {
            value : 0
        }
    }

    pub fn value(&self) -> u64 {
        self.value
    }    

    pub fn address(&self) -> usize {
        // & 0x000ffffffffff000 because address is held in bits 12-52
        self.value as usize & 0x000fffff_fffff000
    }                          

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.value)
    }    

    pub fn set_unused(&mut self) {
        self.value = 0;
    }    

    pub fn is_set(&self) -> bool {
        self.value != 0
    }

    pub fn set_frame(&mut self, frame : Frame, flags : EntryFlags) {
        self.set(frame.address(), flags)
    }    

    pub fn set(&mut self, address : usize, flags : EntryFlags) {        
        assert!(address & !0x000fffff_fffff000 == 0, "Address {} cannot be packed in 52 bits. Table entry value can be maximum 52 bits long", address);
        self.value = (address as u64) | flags.bits();
    }
}

impl fmt::Display for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
            "value: {} , 
            address {}, 
            flags {}", 
            self.value(), 
            self.address(), 
            self.flags().bits())
    }
}

bitflags! {
    pub struct EntryFlags : u64 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const NO_EXECUTE =      1 << 63;
    }
}