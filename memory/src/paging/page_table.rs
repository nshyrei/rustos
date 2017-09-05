use core::marker;
use core::ops;
use frame::Frame;
use frame::frame_allocator::FrameAllocator;
use core::fmt::Display;

pub type VirtualFrame = Frame;

pub type PhysicalFrame = Frame;

pub fn map(page : VirtualFrame, frame : PhysicalFrame, frame_allocator : &mut FrameAllocator) {
    let mut p4 = p4_table();
    let mut p1 = p4.next_table_or_create(page, frame_allocator)
                   .next_table_or_create(page, frame_allocator)
                   .next_table_or_create(page, frame_allocator);

    let p1_index = P1::page_index(page.number());
    p1[p1_index].set_frame(frame, PRESENT)
}

pub fn translate(page : VirtualFrame) -> Option<Frame> {
    let p4 = p4_table();

    p4.next_table_opt(page)
      .and_then(|p3| p3.next_table_opt(page))
      .and_then(|p2| p2.next_table_opt(page))
      .map(|p1| { 
        let p1_index = P1::page_index(page.number());
        let frame_address = p1[p1_index].address();
        Frame::from_address(frame_address)
      })
}

fn p4_table() -> &'static mut PageTable<P4> {
    const P4_TABLE_ADDRESS : usize = 0xfffffffffffff000; //recursive mapping to P4s 0 entry
    unsafe { &mut (*(P4_TABLE_ADDRESS as *mut PageTable<P4>)) }
}

pub fn new_page_table<L>(frame_allocator : &mut FrameAllocator) -> &'static PageTable<L> where L : TableLevel {
    let new_frame = frame_allocator.allocate().expect("No memory for page table");    
    let result = unsafe { &mut (*(new_frame.address() as *mut PageTable<L>)) };
        
    for entry in result.entries.iter_mut() {
        entry.zero();
    };

    result
}

pub trait TableLevel {
    fn index_shift() -> usize;

    fn page_index(index : usize) -> usize {
        index >> Self::index_shift() & 511
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

impl<Level> PageTable<Level> where  Level : HasNextTableLevel {

    pub fn has_next_table(&self, index : usize) -> bool {        
        let table_entry = &self[index];
        let flags = table_entry.flags();

        table_entry.is_present() && flags.contains(PRESENT)
    }    

    fn next_table(&self, index : usize) -> &'static mut PageTable<Level::NextTableLevel> {
        let table_address = self as *const _ as usize;
        let addr = (table_address << 9) | (index << 12);

        unsafe { &mut (*(addr as *mut PageTable<Level::NextTableLevel>)) }  
    }

    pub fn next_table_opt(&self, page : VirtualFrame) -> Option<&'static mut PageTable<Level::NextTableLevel>> {
        let index = Level::page_index(page.number());
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
        let index = Level::page_index(page.number());

        if self.has_next_table(index) {
            self.next_table(index)
        }
        else {
            // create next level table
            let new_page = new_page_table::<Level::NextTableLevel>(frame_allocator);
            let page_address = new_page as *const _ as usize; // basically frame.address()

            // set next level table entry in current table
            self[index].set(page_address, PRESENT);
            
            self.next_table(index)
        }
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

    pub fn zero(&mut self) {
        self.value = 0;
    }

    pub fn address(&self) -> usize {
        // & 0x000ffffffffff000 because address is held in bits 12-52
        (self.value as usize  & 0x000ffffffffff000) >> 12
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
        //assert!(address & 0xffffff0000000000 != 0, "Address {} cannot be packed in 40 bits. Table entry value can be maximum 40 bits long", address);
        self.value = ((address as u64) << 12) | flags.bits();
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
