use memory::kernel::bump_allocator::BumpAllocator;
use memory::kernel::empty_frame_list::{EmptyFrameList, EmptyFrameListIterator};
use memory::frame::Frame;

#[test]
fn adding_elems_should_work_properly() {
    let bytes = [0; 256];
    let addr = bytes.as_ptr() as usize;
    let test_values  = [Frame::new(0), Frame::new(2), Frame::new(3), Frame::new(4), Frame::new(12), Frame::new(20), Frame::new(44), Frame::new(10)];
    let test_values_len = test_values.len();
    let mut KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address(addr);
    let mut head = EmptyFrameList::new_tail(test_values[0], &mut KERNEL_BASIC_HEAP_ALLOCATOR);

    for i in 1..test_values_len {
        head = head.add(test_values[i],&mut KERNEL_BASIC_HEAP_ALLOCATOR);
    }

    let it = EmptyFrameListIterator::new(head);
    let it_count = it.count();

    assert!(it_count == test_values_len,
            "Test values len and returned len aren't equal. Test values len = {}, while returned len = {}",
            test_values_len,
            it_count);

    let mut iterator = EmptyFrameListIterator::new(head);
    let mut idx = test_values_len - 1;
    while let Some(e) = iterator.next() {
        assert!(e == test_values[idx],
                "Test value elem and returned elem aren't equal. Test value = {}, returned value = {}",
                test_values[idx],
                e);

        idx = if idx <= 0 { 0 } else { idx - 1 }; // if idx = 0 it will throw underflow exception, because idx is usize
    }
}