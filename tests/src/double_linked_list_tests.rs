use memory::frame::Frame;
use memory::frame::FRAME_SIZE;
use stdx::iterator::IteratorExt;
use stdx::Iterable;
use stdx::Sequence;
use stdx_memory::MemoryAllocator;
use stdx_memory::heap::SharedBox;
use stdx_memory::collections::double_linked_list::{DoubleLinkedList, DoubleLinkedListIterator};
use memory::allocator::bump::BumpAllocator;
use memory::allocator::bump::ConstSizeBumpAllocator;
use memory::allocator::free_list::FreeListAllocator;
use std::mem;
use alloc::heap;

macro_rules! init_dlist {
    ($l:expr) => {{
        unsafe {
            let size = DoubleLinkedList::<u8>::mem_size_for($l);
            let cell_size = DoubleLinkedList::<u8>::cell_size();
            let heap_addr = heap::allocate_zeroed(size, 2);             
            let mut allocator = ConstSizeBumpAllocator::from_address(heap_addr as usize, size, cell_size);
            let mut list : DoubleLinkedList<u8> = DoubleLinkedList::new();

            (list, allocator)
        }
    }}
}

/*
#[test]
pub fn new_should_create_a_new_cell() {    
    let mut bump_allocator = heap();

    let list = DoubleLinkedListCell::new(10, &mut bump_allocator);

    assert!(list.is_cell(), "DoubleLinkedList::new should return DoubleLinkedList::Cell, but it returned DoubleLinkedList::None");
    assert!(list.value_opt().is_some(), "DoubleLinkedList::new created a cell that doesn't containt a value");

    let value = list.value_opt().unwrap();
    assert!(value == 10, "DoubleLinkedList::new created cell with wrong value, should be {}, but was {}", 10, value);
}

#[test]
pub fn is_cell_should_return_true_for_cell() {    
    let mut bump_allocator = heap();

    let nil = SharedBox::new(DoubleLinkedListCell::Nil, &mut bump_allocator);
    let list = SharedBox::new(DoubleLinkedListCell::Cell { value : 1, prev : nil, next : nil }, &mut bump_allocator);

    assert!(list.is_cell(), "DoubleLinkedListCell::is_cell() returned false for DoubleLinkedListCell::Cell but should be true");    
}

#[test]
pub fn is_cell_should_return_false_for_nil() {    
    let mut bump_allocator = heap();

    let nil : SharedBox<DoubleLinkedListCell<usize>> = SharedBox::new(DoubleLinkedListCell::Nil, &mut bump_allocator);    

    assert!(nil.is_cell() == false, "DoubleLinkedListCell::is_cell() returned true for DoubleLinkedListCell::Nil but should be false");    
}

#[test]
pub fn is_nil_should_return_true_for_nil() {    
    let mut bump_allocator = heap();

    let nil : SharedBox<DoubleLinkedListCell<usize>> = SharedBox::new(DoubleLinkedListCell::Nil, &mut bump_allocator);    

    assert!(nil.is_nil(), "DoubleLinkedListCell::is_nil() returned false for DoubleLinkedListCell::Nil but should be true");    
}

#[test]
pub fn is_nil_should_return_false_for_cell() {    
    let mut bump_allocator = heap();

    let nil = SharedBox::new(DoubleLinkedListCell::Nil, &mut bump_allocator);
    let list = SharedBox::new(DoubleLinkedListCell::Cell { value : 1, prev : nil, next : nil }, &mut bump_allocator);

    assert!(list.is_nil() == false, 
        "DoubleLinkedListCell::is_nil() returned true for DoubleLinkedListCell::Cell but should be false");    
}

#[test]
pub fn is_start_and_is_end_should_return_true_for_new_cell() {    
    let mut bump_allocator = heap();

    let list = DoubleLinkedListCell::new(10, &mut bump_allocator);

    assert!(list.is_start(), "DoubleLinkedList::is_start() should return true for single cell but it returned false");
    assert!(list.is_end(), "DoubleLinkedList::is_end() should return true for single cell but it returned false");    
}

#[test]
pub fn is_start_should_return_true_for_start_cell() {    
    let mut bump_allocator = heap();

    let mut start = DoubleLinkedListCell::new(10, &mut bump_allocator);
    let end = start.add(20, &mut bump_allocator);

    assert!(start.is_start(), "DoubleLinkedList::is_start() should return true for start cell but it returned false");    
}

#[test]
pub fn is_end_should_return_true_for_end_cell() {    
    let mut bump_allocator = heap();

    let mut start = DoubleLinkedListCell::new(10, &mut bump_allocator);
    let end = start.add(20, &mut bump_allocator);

    assert!(end.is_end(), "DoubleLinkedList::is_end() should return true for end cell it returned false");    
}

#[test]
pub fn remove_should_properly_delete_start_element() {  
    use std::mem;  
    let (heap_start, heap_size) = heap_raw!(256);
    let mut bump_allocator = FreeListAllocator::from_address(heap_start, heap_size, mem::size_of::<DoubleLinkedListCell<u8>>());

    let mut start = DoubleLinkedListCell::new(10, &mut bump_allocator);
    let mut mid = start.add(20, &mut bump_allocator);
    let end = mid.add(30, &mut bump_allocator);

    start.remove(&mut bump_allocator);

    assert!(mid.is_start(), 
        "DoubleLinkedList::remove() didn't properly removed start element. Element {} should be the start element but it wasn't",
        mid.value());

    assert!(end.is_end(), 
        "DoubleLinkedList::remove() didn't properly removed start element. Element {} should be the end element but it wasn't",
        end.value());    
}

#[test]
pub fn remove_should_properly_delete_end_element() {
    use std::mem;  
    let (heap_start, heap_size) = heap_raw!(256);
    let mut bump_allocator = FreeListAllocator::from_address(heap_start, heap_size, mem::size_of::<DoubleLinkedListCell<u8>>());

    let mut start = DoubleLinkedListCell::new(10, &mut bump_allocator);
    let mut mid = start.add(20, &mut bump_allocator);
    let mut end = mid.add(30, &mut bump_allocator);

    end.remove(&mut bump_allocator);

    assert!(start.is_start(), 
        "DoubleLinkedList::remove() didn't properly removed end element. Element {} should be the start element but it wasn't",
        start.value());

    assert!(mid.is_end(), 
        "DoubleLinkedList::remove() didn't properly removed end element. Element {} should be the end element but it wasn't",
        end.value());
}
*/

#[test]
pub fn new_should_create_an_empty_list() {    
    let (mut list, mut allocator) = init_dlist!(0);
    
    assert!(list.is_nil(), 
        "DoubleLinkedList::new didn't properly create an empty list. Head or tail wasn't nil");    
}

#[test]
pub fn add_to_tail_should_properly_add_one_cell_to_empty_list() {    
    let (mut list, mut allocator) = init_dlist!(1);
    
    list.add_to_tail(1, &mut allocator);    

    let head = list.head();
    let tail = list.tail();

    assert!(head.is_some(), "DoubleLinkedList::add wasn't able to add cell to empty list. Head was Nil, but should be Cell");
    assert!(tail.is_some(), "DoubleLinkedList::add wasn't to add cell to empty list. Tail was Nil, but should be Cell");
    assert!(head.unwrap() == tail.unwrap(), 
        "DoubleLinkedList::add wasn't able to add cell to empty list. 
        Tail and Head values are different.add_to_tail_should_properly_add_one_cell. Head has value {}, but tail has {}",
        head.unwrap(),
        tail.unwrap());
}

#[test]
pub fn add_to_tail_should_properly_add_cells_to_non_empty_list() {
    let (mut list, mut allocator) = init_dlist!(3);
    let values : [u8;3] = [1, 2, 3];
    let values_len = values.len();

    list.add_to_tail(values[0], &mut allocator);
    list.add_to_tail(values[1], &mut allocator); 
    list.add_to_tail(values[2], &mut allocator);

    assert!(list.is_one_cell() == false, "DoubleLinkedList::add wasn't able to add cell to non empty list. DoubleLinkedList::is_one_cell() returned true");
    
    let count = list.iterator().count();

    assert!(count == values_len, "DoubleLinkedList::add wasn't able to add cell to non empty list. Iterator.count() returned {}, but should be {}",
        count,
        values_len);

    let mut iter = list.iterator().index_items();
    
    // todo : make a reverse indexing iterator
    while let Some((result, index)) = iter.next() {
        assert!(result == values[index], "DoubleLinkedList::add wasn't able to add cell to non empty list. Value returned from iterator and reference differ. Was {}, but should be {}",
        result,
        values[index]);        
    };
}

#[test]
pub fn remove_tail_should_not_do_anything_if_list_is_nil() {
    let (mut list, mut allocator) = init_dlist!(0);

    list.remove_tail(&mut allocator);
    
    assert!(list.is_nil(), "DoubleLinkedList::remove_tail on empty list removed Nil value");        
}

#[test]
pub fn remove_head_should_not_do_anything_if_list_is_nil() {
    let (mut list, mut allocator) = init_dlist!(0);

    list.remove_head(&mut allocator);
    
    assert!(list.is_nil(), "DoubleLinkedList::remove_tail on empty list removed Nil value");        
}

#[test]
pub fn remove_tail_should_properly_remove_tail_element() {
    let (mut list, mut allocator) = init_dlist!(3);

    let values : [u8;3] = [1, 2, 3];
    let values_len = values.len();

    list.add_to_tail(values[0], &mut allocator);
    list.add_to_tail(values[1], &mut allocator); 
    list.add_to_tail(values[2], &mut allocator);

    list.remove_tail(&mut allocator);
    
    let count = list.iterator().count();

    assert!(count == values_len - 1, "DoubleLinkedList::remove_tail didn't remove the last element. Iterator count returned {}, but should be {}",
        count,
        values_len - 1);

    let mut iter = list.iterator().index_items();
    
    // todo : make a reverse indexing iterator
    while let Some((result, index)) = iter.next() {
        assert!(result == values[index], "DoubleLinkedList::remove_tail didn't remove the last element. Value returned from iterator and reference differ. Was {}, but should be {}",
        result,
        values[index]);        
    };      
}

#[test]
pub fn remove_head_should_properly_remove_head_element() {
    let (mut list, mut allocator) = init_dlist!(3);

    let values : [u8;3] = [1, 2, 3];
    let values_len = values.len();

    list.add_to_tail(values[0], &mut allocator);
    list.add_to_tail(values[1], &mut allocator); 
    list.add_to_tail(values[2], &mut allocator);

    list.remove_head(&mut allocator);
    
    let count = list.iterator().count();

    assert!(count == values_len - 1, "DoubleLinkedList::remove_tail didn't remove the last element. Iterator count returned {}, but should be {}",
        count,
        values_len - 1);

    let mut iter = list.iterator().index_items();
        
    while let Some((result, index)) = iter.next() {
        assert!(result == values[index + 1], "DoubleLinkedList::remove_tail didn't remove the last element. Value returned from iterator and reference differ. Was {}, but should be {}",
        result,
        values[index + 1]);
    };      
}

#[test]
pub fn remove_head_should_properly_remove_head_element_when_list_has_exactly_one_cell() {
    let (mut list, mut allocator) = init_dlist!(1);

    list.add_to_tail(1, &mut allocator);
    list.remove_head(&mut allocator);
    
    let count = list.iterator().count();

    assert!(list.is_nil(), "DoubleLinkedList::remove_head didn't remove one existing cell. Is_Nil() returned false but should be true");
}

#[test]
pub fn remove_tail_should_properly_remove_head_element_when_list_has_exactly_one_cell() {
    let (mut list, mut allocator) = init_dlist!(1);

    list.add_to_tail(1, &mut allocator);
    list.remove_tail(&mut allocator);
    
    let count = list.iterator().count();

    assert!(list.is_nil(), "DoubleLinkedList::remove_tail didn't remove one existing cell. Is_Nil() returned false but should be true");
}

#[test]
pub fn take_head_should_return_nothing_if_list_is_nil() {
    let (mut list, mut allocator) = init_dlist!(0);
    let result = list.take_head(&mut allocator);    

    assert!(list.is_nil(), "DoubleLinkedList::take_head returned result from unknown source. Result should be nil because the list is nil");
}

#[test]
pub fn take_head_should_return_one_element_if_list_consists_of_exactly_one_element() {
    let (mut list, mut allocator) = init_dlist!(1);
            
    list.add_to_tail(1, &mut allocator);

    let result = list.take_head(&mut allocator);

    assert!(result.is_some(), "DoubleLinkedList::take_head returned no result, why list had exactly one value {}.",
        1);

    assert!(result.unwrap() == 1, "DoubleLinkedList::take_head returned invalid result, should be {}, but was {}.",
        1,
        result.unwrap());

    assert!(list.is_nil(), "DoubleLinkedList::take_head didn't delete exactly one cell in the list. List is_nil() returned false, but should be true");
}

#[test]
pub fn take_head_should_properly_return_and_then_delete_head_element() {
    
    let (mut list, mut allocator) = init_dlist!(3);
    let values : [u8;3] = [1, 2, 3];    

    list.add_to_tail(values[0], &mut allocator);
    list.add_to_tail(values[1], &mut allocator); 
    list.add_to_tail(values[2], &mut allocator);

    let result     = list.take_head(&mut allocator);    
    let count      = list.iterator().count();
    let values_len = values.len();

    assert!(result.is_some(), "DoubleLinkedList::take_head returned no result, but should return {}",
        values[0]);

    assert!(result.unwrap() == 1, "DoubleLinkedList::take_head returned invalid result, should be {}, but was {}.",
        values[0],
        result.unwrap());

    assert!(count == values_len - 1, "DoubleLinkedList::take_head didn't remove the last element. Iterator count returned {}, but should be {}",
        count,
        values_len - 1);

    let mut iter = list.iterator().index_items();
        
    while let Some((result, index)) = iter.next() {
        assert!(result == values[index + 1], "DoubleLinkedList::take_head didn't remove the last element. Value returned from iterator and reference differ. Was {}, but should be {}",
        result,
        values[index + 1]);
    };
}