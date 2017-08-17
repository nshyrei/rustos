use core::marker;
use core::ops;
use frame::Frame;
use frame::frame_allocator::FrameAllocator;

type VirtualFrame = Frame;
type PhysicalFrame = Frame;

pub fn map(page : VirtualFrame, frame : PhysicalFrame, frame_allocator : &mut FrameAllocator) {
    let mut p4 = p4_table();
    let mut p1 = p4.next_table_or_create(page, frame_allocator)
                   .next_table_or_create(page, frame_allocator)
                   .next_table_or_create(page, frame_allocator);

    let p1_index = P1::page_index(page.number());
    p1[p1_index].set(frame.number(), PRESENT)
}

pub fn map_test(address : usize, page : VirtualFrame, frame : PhysicalFrame, frame_allocator : &mut FrameAllocator, p4_test : &mut TestPageTable<P4>, address_savings : &mut [u64; 3]) {    
    let p4 = p4_test_table(address);
    let mut p1 = p4.next_table_or_create(page, frame_allocator, p4_test, address_savings)
                   .next_table_or_create(page, frame_allocator, p4_test, address_savings)
                   .next_table_or_create(page, frame_allocator, p4_test, address_savings);

    let p1_index = P1::page_index(page.number());
    p1[p1_index].set(frame.number(), PRESENT)
}

pub fn translate_test(address : usize, page : VirtualFrame, p4_test : &mut TestPageTable<P4>, address_savings : &mut [u64; 3]) -> Option<Frame> {
    let p4 = p4_test_table(address);
    p4.next_table_opt(page, p4_test, address_savings)
      .and_then(|p3| p3.next_table_opt(page, p4_test, address_savings))
      .and_then(|p2| p2.next_table_opt(page, p4_test, address_savings))
      .map(|p1| { 
        let p1_index = P1::page_index(page.number());
        let frame_address = p1[p1_index].address();
        Frame::from_address(frame_address)
      })    
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

pub fn p4_test_table(address : usize) -> &'static mut TestPageTable<P4> {
    unsafe { &mut (*(address as *mut TestPageTable<P4>)) }    
}

fn new_page_table<L>(frame_allocator : &mut FrameAllocator) -> &'static PageTable<L> where L : PageLevel {
    let new_frame = frame_allocator.allocate().expect("No memory for page table");
    let result = unsafe { &mut (*(new_frame.address() as *mut PageTable<L>)) };
        
    for entry in result.entries.iter_mut() {
        entry.zero();
    };

    result
}

pub trait PageLevel {
    fn index_shift() -> usize;

    fn page_index(index : usize) -> usize {
        index >> Self::index_shift() & 511
    }
}

pub trait NextPageLevel : PageLevel {
    type NextLevel : PageLevel;    
}

pub trait LevelTestIndex {
    fn address_savings_index() -> usize;
}

pub enum P4 {}
pub enum P3 {}
pub enum P2 {}
pub enum P1 {}

impl PageLevel for P4 {
    fn index_shift() -> usize {
        27
    }
}

impl PageLevel for P3 {
    fn index_shift() -> usize {
        18
    }
}

impl PageLevel for P2 {
    fn index_shift() -> usize {
        9
    }
}

impl PageLevel for P1 {
    fn index_shift() -> usize {
        0
    }
}

impl NextPageLevel for P4 {
    type NextLevel = P3;
}

impl NextPageLevel for P3 {
    type NextLevel = P2;
}

impl NextPageLevel for P2 {
    type NextLevel = P1;
}

impl LevelTestIndex for P4 {
    fn address_savings_index() -> usize {
        0
    }
}

impl LevelTestIndex for P3 {
    fn address_savings_index() -> usize {
        0
    }
}

impl LevelTestIndex for P2 {
    fn address_savings_index() -> usize {
        1
    }
}

impl LevelTestIndex for P1 {
    fn address_savings_index() -> usize {
        2
    }
}

pub struct PageTable<Level : PageLevel> {    
    entries : [PageTableEntry; 512], // 512 * 8 (sizeof(PageTableEntry)) = 4096 b = 4kb = 1 Frame size
                                     // why this size? Because x86-64 spec.
    level : marker::PhantomData<Level>
}

impl<L> ops::Index<usize> for PageTable<L> where L : PageLevel {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &PageTableEntry {        
        &self.entries[index]
    }
}

impl<L> ops::IndexMut<usize> for PageTable<L> where L : PageLevel {
    fn index_mut(&mut self, index : usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

impl<Level> PageTable<Level> where  Level : NextPageLevel {

    pub fn has_next_table(&self, index : usize) -> bool {        
        let table_entry = &self[index]; //todo check if that is indeed ref to entry not ref to ref
        let flags = table_entry.flags();

        table_entry.is_present() && flags.contains(PRESENT)
    }        

    // accessing invalid records in page table will result in page fault
    fn next_table(&self, index : usize) -> &'static mut PageTable<Level::NextLevel> {        
        let table_address = self as *const _ as usize;
        let addr = (table_address << 9) | (index << 12);

        unsafe { &mut (*(addr as *mut PageTable<Level::NextLevel>)) }
    }

    pub fn next_table_opt(&self, page : VirtualFrame) -> Option<&'static mut PageTable<Level::NextLevel>> {
        let index = Level::page_index(page.number());
        if self.has_next_table(index) {
            Some(self.next_table(index))
        }
        else {
            None
        }
    }

    pub fn next_table_or_create(&mut self, page : VirtualFrame, frame_allocator : &mut FrameAllocator) -> &'static mut PageTable<Level::NextLevel> {
        // page number is destructured to check if its index points to 
        // valid (present) page table entry. Recursive looping in P4 table is
        // used to physically address the desired table/frame. 
        let index = Level::page_index(page.number());

        if self.has_next_table(index) {
            self.next_table(index)
        }
        else {
            // create next level page
            let new_page = new_page_table::<Level::NextLevel>(frame_allocator);
            let page_address = new_page as *const _ as usize; // basically frame.address()
            self[index].set(page_address, PRESENT);
            
            self.next_table(index)
        }
    }
}

pub struct TestPageTable<Level : PageLevel> {    
    entries : [PageTableEntry; 512], // 512 * 8 (sizeof(PageTableEntry)) = 4096 b = 4kb = 1 Frame size
                                     // why this size? Because x86-64 spec.    
    level : marker::PhantomData<Level>
}

impl<L> ops::Index<usize> for TestPageTable<L> where L : PageLevel {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &PageTableEntry {        
        &self.entries[index]
    }
}

impl<L> ops::IndexMut<usize> for TestPageTable<L> where L : PageLevel {
    fn index_mut(&mut self, index : usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

impl<Level> TestPageTable<Level> where  Level : NextPageLevel + LevelTestIndex {

    pub fn has_next_table(&self, index : usize) -> bool {        
        let table_entry = &self[index]; //todo check if that is indeed ref to entry not ref to ref
        let flags = table_entry.flags();

        table_entry.is_present() && flags.contains(PRESENT)
    }        

    // test only, don't use in real system
    pub fn next_table(&self, index : usize, p4 : &mut TestPageTable<P4>, address_savings : &mut [u64; 3]) -> &'static mut TestPageTable<Level::NextLevel> {

        let table_address = address_savings[Level::address_savings_index()] as usize;
        let addr = (table_address << 9) | (index << 12);
        address_savings[Level::address_savings_index() + 1] = addr as u64; 
        let destructured_address = self.destructure_address(addr);
        //let p1_index = p4_index1 & 0x0000fffffffff000 >> (0 + 12);

        unsafe {
            let p3 = &(*(p4[destructured_address.0].address() as *const [PageTableEntry; 512]));
            let p2 = &(*(p3[destructured_address.1].address() as *const [PageTableEntry; 512]));
            let p1 = &(*(p2[destructured_address.2].address() as *const [PageTableEntry; 512]));
            let result = &mut (*(p1[destructured_address.3].address() as *mut TestPageTable<Level::NextLevel>));
        
            result
        }        
    }

    fn destructure_address(&self, addr : usize) -> (usize, usize, usize, usize) {
        let p4_index = (addr & 0x0000fffffffff000) >> (27 + 12) & 511;
        let p3_index = (addr & 0x0000fffffffff000) >> (18 + 12) & 511;
        let p2_index = (addr & 0x0000fffffffff000) >> (9 + 12) & 511;
        let p1_index = (addr & 0x0000fffffffff000) >> (0 + 12) & 511;

        (p4_index, p3_index, p2_index, p1_index)
    }

    pub fn next_table_or_create(&mut self, page : VirtualFrame, frame_allocator : &mut FrameAllocator, p4 : &mut TestPageTable<P4>, address_savings : &mut [u64; 3]) -> &'static mut TestPageTable<Level::NextLevel> {
        // page number is destructured to check if its index points to 
        // valid (present) page table entry. Recursive looping in P4 table is
        // used to physically address the desired table/frame. 
        let index = Level::page_index(page.number());

        if p4.has_next_table(index) {
            self.next_table(index, p4, address_savings)
        }
        else {
            // create next level page
            let new_page = new_page_table::<Level::NextLevel>(frame_allocator);
            let page_address = new_page as *const _ as usize; // basically frame.address()
            p4[index].set(page_address, PRESENT);
            
            self.next_table(index, p4, address_savings)
        }
    }
    
    pub fn next_table_opt(&self, page : VirtualFrame, p4 : &mut TestPageTable<P4>, address_savings : &mut [u64; 3]) -> Option<&'static mut TestPageTable<Level::NextLevel>> {
        let index = Level::page_index(page.number());
        if self.has_next_table(index) {
            Some(self.next_table(index, p4, address_savings))
        }
        else {
            None
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
        // todo check if address is lover then 40bits

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