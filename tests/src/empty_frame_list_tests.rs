use memory::util::bump_allocator::BumpAllocator;
use memory::util::linked_list::{LinkedList, LinkedListIterator};
use memory::frame::Frame;
use memory::frame::FRAME_SIZE;

#[test]
fn adding_elems_should_work_properly() {
    let bytes = [0; 256];
    let addr = bytes.as_ptr() as usize;
    let test_values  = [
        Frame::from_address(0), 
        Frame::from_address(FRAME_SIZE * 2), 
        Frame::from_address(FRAME_SIZE * 3),
        Frame::from_address(FRAME_SIZE * 4), 
        Frame::from_address(FRAME_SIZE * 12), 
        Frame::from_address(FRAME_SIZE * 20), 
        Frame::from_address(FRAME_SIZE * 44), 
        Frame::from_address(FRAME_SIZE * 10)
    ];
    let test_values_len = test_values.len();
    let mut KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address(addr, 256);
    let mut head = LinkedList::new(test_values[0], &mut KERNEL_BASIC_HEAP_ALLOCATOR);

    for i in 1..test_values_len {
        head = head.pointer().add(test_values[i],&mut KERNEL_BASIC_HEAP_ALLOCATOR);
    }

    let it = LinkedListIterator::new(head);
    let it_count = it.count();

    assert!(it_count == test_values_len,
            "Test values len and returned len aren't equal. Test values len = {}, while returned len = {}",
            test_values_len,
            it_count);

    let mut iterator = LinkedListIterator::new(head);
    let mut idx = test_values_len - 1;
    while let Some(e) = iterator.next() {
        assert!(e == test_values[idx],
                "Test value elem and returned elem aren't equal. Test value = {}, returned value = {}",
                test_values[idx],
                e);

        idx = if idx <= 0 { 0 } else { idx - 1 }; // if idx = 0 it will throw underflow exception, because idx is usize
    }
}