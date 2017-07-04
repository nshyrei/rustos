use memory::kernel::bump_allocator::BumpAllocator;
use memory::kernel::stack_list::{EmptyFrameList, EmptyFrameListIterator};


#[test]
fn adding_elems_should_work_properly() {
    let bytes = [0; 256];
    let addr = bytes.as_ptr() as usize;
    let test_values = [0, 2, 3, 4, 12, 20, 44, 10];
    let test_values_len = test_values.len();

    let mut allocator = BumpAllocator::from_address(addr);
    let mut head = EmptyFrameList::new(test_values[0], &mut allocator);

    for i in 1..test_values_len {
        head = head.add(test_values[i], &mut allocator);
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