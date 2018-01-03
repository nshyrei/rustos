use memory::frame::Frame;
use memory::frame::FRAME_SIZE;
use stdx_memory::MemoryAllocator;
use stdx_memory::heap::SharedBox;
use stdx_memory::collections::double_linked_list::{DoubleLinkedListCell, DoubleLinkedList};
use memory::allocator::bump::BumpAllocator;
use memory::allocator::free_list::FreeListAllocator;


fn heap() -> BumpAllocator {
    let heap = [0;256];
    let heap_addr = heap.as_ptr() as usize;
    BumpAllocator::from_address(heap_addr, heap.len())
}

macro_rules! heap_raw {
    () => {{
        let heap = [0;256];    

        (heap.as_ptr() as usize, 256)
    }}    
}

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
    let (heap_start, heap_size) = heap_raw!();
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
    let (heap_start, heap_size) = heap_raw!();
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