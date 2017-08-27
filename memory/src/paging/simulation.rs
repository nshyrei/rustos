use core::marker;
use core::ops;
use paging::page_table::*;
use frame::Frame;
use frame::frame_allocator::FrameAllocator;

pub fn map(p4 : &mut TestPageTable<P4>, page : VirtualFrame, frame : PhysicalFrame, frame_allocator : &mut FrameAllocator, paging_simulation : &mut PagingSimulation) {    
    
    let mut p1 = p4.next_table_or_create(page, frame_allocator, paging_simulation)
                   .next_table_or_create(page, frame_allocator, paging_simulation)
                   .next_table_or_create(page, frame_allocator, paging_simulation);

    let p1_index = P1::page_index(page.number());
    p1[p1_index].set_frame(frame, PRESENT)
}

pub fn translate(p4 : &mut TestPageTable<P4>, page : VirtualFrame, paging_simulation : &mut PagingSimulation) -> Option<Frame> {    
    p4.next_table_opt(page, paging_simulation)
      .and_then(|p3| p3.next_table_opt(page, paging_simulation))
      .and_then(|p2| p2.next_table_opt(page, paging_simulation))
      .map(|p1| { 
        let p1_index = P1::page_index(page.number());
        let frame_address = p1[p1_index].address();
        Frame::from_address(frame_address)
      })    
}

pub trait AddressSavingsIndex {
    fn address_savings_index() -> usize;
}

impl AddressSavingsIndex for P4 {
    fn address_savings_index() -> usize {
        0
    }
}

impl AddressSavingsIndex for P3 {
    fn address_savings_index() -> usize {
        0
    }
}

impl AddressSavingsIndex for P2 {
    fn address_savings_index() -> usize {
        1
    }
}

impl AddressSavingsIndex for P1 {
    fn address_savings_index() -> usize {
        2
    }
}

pub struct TestPageTable<Level> where Level : TableLevel {    
    entries : [PageTableEntry; 512], // 512 * 8 (sizeof(PageTableEntry)) = 4096 b = 4kb = 1 Frame size
                                     // why this size? Because x86-64 spec.    
    level : marker::PhantomData<Level>
}

impl<L> ops::Index<usize> for TestPageTable<L> where L : TableLevel {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &PageTableEntry {        
        &self.entries[index]
    }
}

impl<L> ops::IndexMut<usize> for TestPageTable<L> where L : TableLevel {
    fn index_mut(&mut self, index : usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

impl<Level> TestPageTable<Level> where Level : HasNextTableLevel + AddressSavingsIndex {

    pub fn has_next_table(&self, index : usize) -> bool {        
        let table_entry = &self[index];
        let flags = table_entry.flags();

        table_entry.is_present() && flags.contains(PRESENT)
    }    

    pub fn next_table_or_create(&mut self, page : VirtualFrame, frame_allocator : &mut FrameAllocator, paging_simulation : &mut PagingSimulation) -> &'static mut TestPageTable<Level::NextTableLevel> {
        // page number is destructured to check if its index points to 
        // valid (present) page table entry. Recursive looping in P4 table is
        // used to physically address the desired table/frame. 
        let index = Level::page_index(page.number());

        if self.has_next_table(index) {
            paging_simulation.next_table::<Level>(index)            
        }
        else {
            // create next level page
            let new_page = new_page_table::<Level::NextTableLevel>(frame_allocator);
            let page_address = new_page as *const _ as usize; // basically frame.address()
            self[index].set(page_address, PRESENT);
            
            paging_simulation.next_table::<Level>(index)
        }
    }
    
    pub fn next_table_opt(&self, page : VirtualFrame, paging_simulation : &mut PagingSimulation) -> Option<&'static mut TestPageTable<Level::NextTableLevel>> {
        let index = Level::page_index(page.number());
        if self.has_next_table(index) {
            Some(paging_simulation.next_table::<Level>(index))
        }
        else {
            None
        }
    }
}

pub struct PagingSimulation {
    p4_address : usize, 
    address_savings : [u64; 3]  // holds previous address, 0 entry for P4 table and up to P2
}

impl PagingSimulation {

    pub fn new(p4_address : usize) -> PagingSimulation {
        let mut address_savings : [u64; 3] = [0; 3];
        address_savings[0] = 0x0000fffffffff000;
        PagingSimulation {
            p4_address : p4_address,
            address_savings : address_savings
        }
    }

    pub fn next_table<Level>(&mut self, index : usize) -> &'static mut TestPageTable<Level::NextTableLevel>
    where Level : HasNextTableLevel + AddressSavingsIndex   {
        
        let table_address = self.address_savings[Level::address_savings_index()] as usize;
        let addr = (table_address << 9) | (index << 12);
        self.address_savings[Level::address_savings_index() + 1] = addr as u64; 
        let destructured_address = PagingSimulation::destructure_address(addr);
        
        unsafe {
            let p4 = &(*(self.p4_address as *const TestPageTable<P4>));
            let p3 = &(*(p4[destructured_address.0].address() as *const [PageTableEntry; 512]));
            let p2 = &(*(p3[destructured_address.1].address() as *const [PageTableEntry; 512]));
            let p1 = &(*(p2[destructured_address.2].address() as *const [PageTableEntry; 512]));
            let result = &mut (*(p1[destructured_address.3].address() as *mut TestPageTable<Level::NextTableLevel>));
        
            result
        }        
    }

    fn destructure_address(addr : usize) -> (usize, usize, usize, usize) {
        let p4_index = (addr & 0x0000fffffffff000) >> (27 + 12) & 511;
        let p3_index = (addr & 0x0000fffffffff000) >> (18 + 12) & 511;
        let p2_index = (addr & 0x0000fffffffff000) >> (9 + 12) & 511;
        let p1_index = (addr & 0x0000fffffffff000) >> (0 + 12) & 511;

        (p4_index, p3_index, p2_index, p1_index)
    }

    pub fn reset_address_savings(&mut self) {
        for i in 0..self.address_savings.len(){
            self.address_savings[i] = 0;
        }
        self.address_savings[0] = 0x0000fffffffff000;
    }
}