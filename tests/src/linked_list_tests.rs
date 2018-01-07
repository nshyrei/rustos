use memory::allocator::bump::BumpAllocator;
use stdx_memory::collections::linked_list::{LinkedList, LinkedListIterator};
use stdx_memory::heap::SharedBox;
use memory::frame::Frame;
use memory::frame::FRAME_SIZE;

fn heap() -> BumpAllocator {
    let heap = [0;256];
    let heap_addr = heap.as_ptr() as usize;        
    BumpAllocator::from_address_for_type::<LinkedList<u64>>(heap_addr, heap.len())
}

#[test]
pub fn new_should_create_a_new_cell() {    
    let mut bump_allocator = heap();

    let list = LinkedList::new(10, &mut bump_allocator);

    assert!(list.is_cell(), "LinkedList::new should return LinkedList::Cell, but it returned LinkedList::None");
    assert!(list.value().is_some(), "LinkedList::new created a cell that doesn't containt a value");

    let value = list.value().unwrap();
    assert!(value == 10, "LinkedList::new created cell with wrong value, should be {}, but was {}", 10, value);
}

#[test]
pub fn is_cell_should_return_true_for_cell() {    
    let mut bump_allocator = heap();

    let nil = SharedBox::new(LinkedList::Nil, &mut bump_allocator);
    let list = SharedBox::new(LinkedList::Cell { value : 1, prev : nil }, &mut bump_allocator);

    assert!(list.is_cell(), "LinkedList::is_cell() returned false for LinkedList::Cell but should be true");    
}

#[test]
pub fn is_cell_should_return_false_for_nil() {    
    let mut bump_allocator = heap();

    let nil : SharedBox<LinkedList<usize>> = SharedBox::new(LinkedList::Nil, &mut bump_allocator);    

    assert!(nil.is_cell() == false, "LinkedList::is_cell() returned true for LinkedList::Nil but should be false");    
}

#[test]
pub fn is_nil_should_return_true_for_nil() {    
    let mut bump_allocator = heap();

    let nil : SharedBox<LinkedList<usize>> = SharedBox::new(LinkedList::Nil, &mut bump_allocator);    

    assert!(nil.is_nil(), "LinkedList::is_nil() returned false for LinkedList::Nil but should be true");    
}

#[test]
pub fn is_nil_should_return_false_for_cell() {    
    let mut bump_allocator = heap();

    let nil = SharedBox::new(LinkedList::Nil, &mut bump_allocator);
    let list = SharedBox::new(LinkedList::Cell { value : 1, prev : nil }, &mut bump_allocator);

    assert!(list.is_nil() == false, 
        "LinkedList::is_nil() returned true for LinkedList::Cell but should be false");    
}

#[test]
pub fn add_should_create_a_new_cell_with_reference_to_the_old_one() {    
    let mut bump_allocator = heap();

    let start = LinkedList::new(1, &mut bump_allocator);
    let end = start.add(2, &mut bump_allocator);
    let mut iterator = LinkedListIterator::new(end);

    let fst = iterator.next();
    let snd = iterator.next(); 

    assert!(fst.is_some() && snd.is_some(), 
        "LinkedList::add failed to create new cell that references previous cell.");
    
    let fst_value = fst.unwrap();
    let snd_value = snd.unwrap();

    assert!(fst_value == 2, "LinkedList::add returned cell with invalid value, was {}, but should be {}", fst_value, 2);
    assert!(snd_value == 1, 
        "LinkedList::add returned cell that has invalid reference for previous cell. Previous cell has value {}, but should be {}", 
        snd_value, 1);
}

#[test]
pub fn add_should_create_a_new_cell_with_reference_to_the_old_one_nil_case() {    
    let mut bump_allocator = heap();

    let start : SharedBox<LinkedList<u32>> = SharedBox::new(LinkedList::Nil, &mut bump_allocator);    
    let end = start.add(2, &mut bump_allocator);
    let mut iterator = LinkedListIterator::new(end);

    let fst = iterator.next();
    let snd = iterator.next(); 

    assert!(fst.is_some(), 
        "LinkedList::add failed to create new cell that references previous cell.");
    
    let fst_value = fst.unwrap();    

    assert!(fst_value == 2, "LinkedList::add returned cell with invalid value, was {}, but should be {}", fst_value, 2);
    assert!(snd.is_none(), 
        "LinkedList::add returned cell that has invalid reference for previous cell. Previous cell is Cell but should be Nil");
}

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
    let mut KERNEL_BASIC_HEAP_ALLOCATOR = BumpAllocator::from_address_for_type::<LinkedList<Frame>>(addr, 256);
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