use memory::paging::page_table::*;
use memory::frame::Frame;
use memory::kernel::bump_allocator::BumpAllocator;

#[test]
fn adding_elems_should_work_properly() {
    let mut p4 = [0; 4096];
    let mut p3 = [0; 4096];
    let mut p2 = [0; 4096];
    let mut p1 = [0; 4096];
    let kernel_heap = [0;256];    
    let kernel_heap_addr = kernel_heap.as_ptr() as usize;
        
    let p4_address = p4.as_ptr() as usize;
    p4[0] = p4_address; // set recursive entry

    
    let mut bump_allocator = BumpAllocator::from_address(kernel_heap_addr);


    map(Frame::new(1), Frame::new(2), bump_allocator);
}