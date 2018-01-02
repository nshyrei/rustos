use core::mem;
use util::bump_allocator::BumpAllocator;
use allocator::MemoryAllocator;
use util::{Box, SharedBox};
use util::array::Array;
use frame::Frame;


pub struct BuddyFreeList {
    frame_to_free_buddy : Array<SharedBox<DoubleLinkedListCell<usize>>>,
    free_blocks         : DoubleLinkedList<usize>
}

impl BuddyFreeList {
    pub fn new(frame_count : usize, memory_allocator : &mut BumpAllocator) -> Self {        
        BuddyFreeList {
            frame_to_free_buddy : Array::new(frame_count, memory_allocator),
            free_blocks         : DoubleLinkedList::new(memory_allocator),            
        }
    }

    /// Determines if block is free to use
    /// # Arguments    
    /// * `block_start_address` - start address of memory block
    pub fn is_free(&self, block_start_address : usize) -> bool {
        !self.is_in_use(block_start_address)
    }

    /// Determines if block is occupied
    /// # Arguments    
    /// * `block_start_address` - start address of memory block
    pub fn is_in_use(&self, block_start_address : usize) -> bool {
        // todo block_start_address or frame number will be out of range
        self.frame_to_free_buddy.elem_ref(block_start_address).is_nil()
    }

    /// Sets the block as occupied
    /// # Arguments    
    /// * `block_start_address` - start address of memory block
    /// * `memory_allocator` - memory allocator
    pub fn set_in_use(&mut self, block_start_address : usize, memory_allocator : &mut BumpAllocator) {
        if self.is_free(block_start_address) {
            let cell = self.frame_to_free_buddy.value(block_start_address);
            self.remove_free_block(cell, memory_allocator);
            self.frame_to_free_buddy.update(block_start_address, SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator));        
        }
    }

    /// Sets the block as free to use
    /// # Arguments    
    /// * `block_start_address` - start address of memory block
    /// * `memory_allocator` - memory allocator
    pub fn set_free(&mut self, block_start_address : usize, memory_allocator : &mut BumpAllocator) {
        if self.is_in_use(block_start_address) {
            let cell = self.free_blocks.add_to_tail(block_start_address, memory_allocator);            
            self.frame_to_free_buddy.update(block_start_address, cell);        
        }
    }

    /// Returns first unused memory block if any.
    /// # Arguments        
    /// * `memory_allocator` - memory allocator
    pub fn first_free_block(&mut self, memory_allocator : &mut BumpAllocator) -> Option<usize> {
        let result = self.free_blocks.take_head(memory_allocator);

        if let Some(index) = result {
            self.frame_to_free_buddy.update(index, SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator));
        }

        result
    }

    fn remove_free_block(&mut self, cell : SharedBox<DoubleLinkedListCell<usize>>, memory_allocator : &mut BumpAllocator) {                
        if self.free_blocks.head_equals_tail() && cell.is_start() {
            self.free_blocks.remove_head(memory_allocator);            
        }
        else if cell.is_start() {
            self.free_blocks.remove_head(memory_allocator);            
        }
        else if cell.is_end() {
            self.free_blocks.remove_tail(memory_allocator);            
        }
        else {
            cell.pointer_mut().remove(memory_allocator);
        }
    }
}

pub struct DoubleLinkedList<T> {    
    head : SharedBox<DoubleLinkedListCell<T>>,
    tail : SharedBox<DoubleLinkedListCell<T>>
}

impl<T> DoubleLinkedList<T> {

    /// Creates new Empty DoubleLinkedList
    /// # Arguments    
    /// * `memory_allocator` - memory allocator    
    pub fn new(memory_allocator : &mut BumpAllocator) -> Self {
        DoubleLinkedList {            
            head : SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator),
            tail : SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator)
        }
    }
    
    /// Adds new DoubleLinkedListCell::Cell to the back of `self.tail`
    /// # Arguments
    /// * `value` - value to add
    /// * `memory_allocator` - memory allocator    
    pub fn add_to_tail(&mut self, value : T, memory_allocator : &mut BumpAllocator) -> SharedBox<DoubleLinkedListCell<T>> {
        let new_cell = self.tail.add(value, memory_allocator);

        self.tail = new_cell;

        if self.head.is_nil() {
            self.head = new_cell;
        }

        new_cell
    }
        
    pub fn head(&self) -> SharedBox<DoubleLinkedListCell<T>> {
        self.head
    }

    pub fn tail(&self) -> SharedBox<DoubleLinkedListCell<T>> {
        self.tail
    }

    /// Determines if this linked list consists only of DoubleLinkedListCell::Nil    
    pub fn is_nil(&self) -> bool {
        self.head.is_nil() && self.tail.is_nil()
    }

    /// Determines if this linked list consists only of one DoubleLinkedListCell::Cell
    pub fn is_one_cell(&self) -> bool {
        self.head.is_end() && self.tail.is_start()
    }

    pub fn head_equals_tail(&self) -> bool {
        // head is equal to tail in two cases:
        // 1: they are both pointing to DoubleLinkedList::Nil
        // 2: DoubleLinkedList::is_end() is true for `self.head` (start cell is also a end cell) and
        //    DoubleLinkedList:is_start() is true for `self.tail` (end cell is also a start cell)
        self.is_nil() || self.is_one_cell()
    }

    /// # Arguments    
    /// * `memory_allocator` - memory allocator    
    pub fn remove_head(&mut self, memory_allocator : &mut BumpAllocator) {
        // calling this before self.head.take_next is important to
        // prevent reading freed memory!
        let head_equals_tail = self.head_equals_tail();
        let result = self.head.remove_next(memory_allocator);

        if let Some(new_head) = result {
            if head_equals_tail {
                self.head = new_head;
                self.tail = new_head;
            }
            else {
                self.head = new_head;
            }
        }
    }
    
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    pub fn remove_tail(&mut self, memory_allocator : &mut BumpAllocator) {
        // calling this before self.head.take_next is important to
        // prevent reading freed memory!
        let head_equals_tail = self.head_equals_tail();
        let result = self.head.remove_prev(memory_allocator);

        if let Some(new_tail) = result {
            if head_equals_tail {
                self.head = new_tail;
                self.tail = new_tail;
            }
            else {
                self.tail = new_tail;
            }
        }
    }
}

impl<T> DoubleLinkedList<T> where T : Copy {
    
    /// Deletes current `self.head` from memory and returns copy of its data if it was DoubleLinkedList::Cell.
    /// Returns None otherwise.
    /// # Arguments    
    /// * `memory_allocator` - memory allocator
    pub fn take_head(&mut self, memory_allocator : &mut BumpAllocator) -> Option<T> {
        // calling this before self.head.take_next is important to
        // prevent reading freed memory!
        let head_equals_tail = self.head_equals_tail();
        let result = self.head.take_next(memory_allocator);

        if let Some((_, new_head)) = result {
            if head_equals_tail {
                self.head = new_head;
                self.tail = new_head;
            }
            else {
                self.head = new_head;
            }
        }

        result.map(|e| e.0)
    } 
}

/// A type that represents double linked list of cells in memory
#[repr(C)]
pub enum DoubleLinkedListCell<T> {
    /// Type that represents list start point and end points. Used as a marker and doesn't hold any value. 
    Nil,
    /// Represents list cell that holds value of type `T` and has reference to previous and next DoubleLinkedList
    Cell { value : T, prev : SharedBox<DoubleLinkedListCell<T>>, next : SharedBox<DoubleLinkedListCell<T>> }
}

impl<T> DoubleLinkedListCell<T> {

    /// Creates a new cell, which has `prev` and `next` pointing to DoubleLinkedList::Nil.
    /// # Arguments
    /// * `value` - value to put into cell
    /// * `memory_allocator` - memory allocator
    pub fn new(value: T, memory_allocator : &mut BumpAllocator) -> SharedBox<Self> {        
        let new_cell = DoubleLinkedListCell::Cell {
                value : value,
                next  : SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator),
                prev  : SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator)
        };

        SharedBox::new(new_cell, memory_allocator)        
    }

    /// Creates a new cell, which has `prev` pointing to `self` e.g. previous cell and `next`
    /// pointing to DoubleLinkedList::Nil.
    /// # Arguments
    /// * `value` - value to put into cell
    /// * `memory_allocator` - memory allocator
    pub fn add(&mut self, value: T, memory_allocator : &mut BumpAllocator) -> SharedBox<Self> {
        let nil      = SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator);
        let new_cell = DoubleLinkedListCell::Cell {
                value : value,
                next  : nil,
                prev  : SharedBox::from_pointer(self)
        };

        let result = SharedBox::new(new_cell, memory_allocator);

        self.set_next(result);
        result
    }

    /// Determines if this type is DoubleLinkedList::Cell
    pub fn is_cell(&self) -> bool {
        match *self {
            DoubleLinkedListCell::Cell { .. } => true,
            _ => false
        }
    }

    /// Determines if this type is DoubleLinkedList::Cell which has `prev` pointing to DoubleLinkedList::Nil
    pub fn is_start(&self) -> bool {
        match *self {
            DoubleLinkedListCell::Cell { prev, .. } => prev.is_nil(),
            _ => false
        }
    }

    /// Determines if this type is DoubleLinkedList::Cell which has `next` pointing to DoubleLinkedList::Nil
    pub fn is_end(&self) -> bool {
        match *self {
            DoubleLinkedListCell::Cell { next, .. } => next.is_nil(),
            _ => false
        }
    }

    /// Determines if this type is DoubleLinkedList::Nil
    pub fn is_nil(&self) -> bool {
        !self.is_cell()
    }

    /// Comment out reason : unused, but can be usefull in the future
    /*
    pub fn value_ref_opt(&self) -> Option<&T> {
        match *self {
            DoubleLinkedListCell::Cell { ref value, .. } => Some(value),
            _ => None
        }
    }

    pub fn value_ref(&self) -> &T {
        DoubleLinkedListCell::value_ref_opt(&self).expect("Trying to get value From Nil")
    }

    pub fn next_opt(&self) -> Option<&Self> {
        match *self {
            DoubleLinkedListCell::Cell { ref next, .. } => Some(next.pointer()),
            _ => None
        }
    }

    pub fn prev_opt(&self) -> Option<&Self> {
        match *self {
            DoubleLinkedListCell::Cell { ref prev, .. } => Some(prev.pointer()),
            _ => None
        }
    }
    */

    /// Sets `next` to `new_next` if this type is DoubleLinkedList::Cell.
    /// Does nothing otherwise.
    /// # Arguments
    /// * `new_next` - new pointer to next DoubleLinkedList    
    pub fn set_next(&mut self, new_next : SharedBox<Self>) {
        if let DoubleLinkedListCell::Cell { ref mut next, .. } = *self {
            *next = new_next
        }
    }

    /// Sets `prev` to `new_prev` if this type is DoubleLinkedList::Cell.
    /// Does nothing otherwise.
    /// # Arguments
    /// * `new_prev` - new pointer to previous DoubleLinkedList
    pub fn set_prev(&mut self, new_prev : SharedBox<Self>) {
        if let DoubleLinkedListCell::Cell { ref mut prev, .. } = *self {
            *prev = new_prev
        }    
    }

    /// Deletes this DoubleLinkedList from memory. Returns `prev` and `next` pointers if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.    
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove(&mut self, memory_allocator : &mut BumpAllocator) -> Option<(SharedBox<Self>, SharedBox<Self>)> {
        let result = match *self {
            DoubleLinkedListCell::Cell { mut prev, mut next, .. } => {
                prev.set_next(next);
                next.set_prev(prev);
                Some((prev, next))
            },
            _ => None
        };

        memory_allocator.free(&self as *const _ as usize);
        result
    }

    /// Deletes this DoubleLinkedList from memory. Returns `prev` pointer if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove_prev(&mut self, memory_allocator : &mut BumpAllocator) -> Option<SharedBox<Self>> {
        self.remove(memory_allocator).map(|e| e.0)
    }

    /// Deletes this DoubleLinkedList from memory. Returns `next` pointer if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove_next(&mut self, memory_allocator : &mut BumpAllocator) -> Option<SharedBox<Self>> {
        self.remove(memory_allocator).map(|e| e.1)
    }
}

impl<T> DoubleLinkedListCell<T> where T : Copy {

    /// Returns copy of the value in the cell if `self` is DoubleLinkedList::Cell,
    /// otherwise returns None
    pub fn value_opt(&self) -> Option<T> {
        match *self {
            DoubleLinkedListCell::Cell { value, .. } => Some(value),
            _ => None
        }
    }
}

impl <T> DoubleLinkedListCell<T> where T : Copy {
    /// Returns copy of the cell data if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take(&self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<Self>, SharedBox<Self>)> {        
        let result = match *self {
            DoubleLinkedListCell::Cell { value, mut prev, mut next } => {
                next.set_prev(prev);
                prev.set_next(next);
                
                Some((value, prev, next))
            },
            _ => None
        };

        memory_allocator.free(&self as *const _ as usize);
        result
    }

    /// Returns copy of the cell data and pointer to previous DoubleLinkedList
    /// if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take_prev(&self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<Self>)> {
        self.take(memory_allocator).map(|e| {
            let (value, prev, _) = e;
            (value, prev)
        })
    }

    /// Returns copy of the cell data and pointer to next DoubleLinkedList
    /// if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take_next(&self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<Self>)> {
        self.take(memory_allocator).map(|e| {
            let (value, _, next) = e;
            (value, next)
        })
    }
}