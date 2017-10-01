use core::marker;
use core::ops;
use core::fmt;
use frame::Frame;
use frame::FRAME_SIZE;
use frame::frame_allocator::FrameAllocator;
use hardware::x86_64::tlb;

pub type VirtualFrame = Frame;

pub type PhysicalFrame = Frame;

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
    entries : [PageTableEntry; 512], // 512 * 8 (sizeof(PageTableEntry)) = 4096 b = 4kb = 1 Frame size
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
    fn clear_all_entries(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        };
    }
}

impl<Level> PageTable<Level> where  Level : HasNextTableLevel {

    pub fn has_next_table(&self, index : usize) -> bool {        
        let table_entry = &self[index];
        let flags = table_entry.flags();

        table_entry.is_present() && flags.contains(PRESENT)
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

    pub fn next_table_or_create(&mut self, page : VirtualFrame, frame_allocator : &mut FrameAllocator) -> &'static mut PageTable<Level::NextTableLevel> {
        // page number is destructured to check if its index points to 
        // valid (present) page table entry. Recursive looping in P4 table is
        // used to physically address the desired table/frame. 
        let index = Level::page_index(page);

        if self.has_next_table(index) {
            self.next_table(index)
        }
        else {
            // create next level table
            let new_table_frame = frame_allocator.allocate().expect("No memory for page table");                        

            // set new entry in current table
            self[index].set_frame(new_table_frame, PRESENT | WRITABLE);
            
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

    /// maps virtual page to physical frame
    ///
    /// # Arguments
    /// * `page` - virtual frame
    /// * `frame` - physical frame
    /// * `frame_allocator` - frame allocator
    pub fn map(&mut self, page : VirtualFrame, frame : PhysicalFrame, flags : EntryFlags, frame_allocator : &mut FrameAllocator) {        
        let p1 = self.next_table_or_create(page, frame_allocator)
                         .next_table_or_create(page, frame_allocator)
                         .next_table_or_create(page, frame_allocator);

        let p1_index = P1::page_index(page);
        p1[p1_index].set_frame(frame, flags)
    }


    /// Maps virtual page to physical frame in 1 to 1 fashion, e.g.
    /// virtual page will correspond to physical frame with the same address
    ///
    /// # Arguments
    /// * `page` - virtual frame
    /// * `frame_allocator` - frame allocator
    pub fn map_1_to_1(&mut self, page : VirtualFrame, flags : EntryFlags, frame_allocator : &mut FrameAllocator) {
        let frame = page.clone();
        self.map(page, frame, flags, frame_allocator);
    }

    /// Unmaps virtual page
    ///
    /// # Arguments
    /// * `page` - virtual frame
    /// * `frame_allocator` - frame allocator
    pub fn unmap(&self, page : VirtualFrame) {
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

            if p1_entry.is_present() {            
                Some(Frame::from_address(p1_entry.address()))
            }
            else {
                None
            }        
        })
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
        self.value as usize  & 0x000ffffffffff000
    }    

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.value)
    }

    pub fn is_present(&self) -> bool {
        // todo probably add check for PRESENT flag
        self.value != 0
    }

    pub fn set_unused(&mut self) {
        self.value = 0;
    }    

    pub fn set_frame(&mut self, frame : Frame, flags : EntryFlags) {
        self.set(frame.address(), flags)
    }    

    pub fn set(&mut self, address : usize, flags : EntryFlags) {        
        assert!(address & !0x000fffff_fffff000 == 0, "Address {} cannot be packed in 52 bits. Table entry value can be maximum 40 bits long", address);
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