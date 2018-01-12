use MemoryAllocator;
use heap;
use stdx::iterator;
use core::iter;
use core::mem;

pub struct DoubleLinkedList<T> {
    head : heap::SharedBox<DoubleLinkedListCell<T>>,
    tail : heap::SharedBox<DoubleLinkedListCell<T>>,    
}

impl<T> DoubleLinkedList<T> {

    /// Creates new Empty DoubleLinkedList
    /// # Arguments    
    /// * `memory_allocator` - memory allocator
    pub fn new<A>(memory_allocator : &mut A) -> Self where A : MemoryAllocator
    {
        DoubleLinkedList {            
            head : heap::SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator),
            tail : heap::SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator)
        }
    }
    
    /// Adds new DoubleLinkedListCell::Cell to the back of `self.tail`
    /// # Arguments
    /// * `value` - value to add
    /// * `memory_allocator` - memory allocator    
    pub fn add_to_tail<A>(&mut self, value : T, memory_allocator : &mut A) -> heap::SharedBox<DoubleLinkedListCell<T>> where A : MemoryAllocator{
        let new_cell = self.tail.add(value, memory_allocator);

        self.tail = new_cell;

        if self.head.is_nil() {
            self.head = new_cell;
        }

        new_cell
    }
        
    pub fn head(&self) -> heap::SharedBox<DoubleLinkedListCell<T>> {
        self.head
    }

    pub fn tail(&self) -> heap::SharedBox<DoubleLinkedListCell<T>> {
        self.tail
    }

    /// Determines if this linked list consists only of DoubleLinkedListCell::Nil    
    pub fn is_nil(&self) -> bool {
        self.head.is_nil() && self.tail.is_nil()
    }

    /// Determines if this linked list contains any DoubleLinkedListCell::Cell
    pub fn is_cell(&self) -> bool {
        !self.is_nil()
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
    pub fn remove_head<A>(&mut self, memory_allocator : &mut A) where A : MemoryAllocator {
        // calling this before self.head.take_next is important to
        // prevent reading freed memory!
        if !self.is_nil() {
            let head_equals_tail = self.is_one_cell();
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
    }
    
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    pub fn remove_tail<A>(&mut self, memory_allocator : &mut A) where A : MemoryAllocator{
        if !self.is_nil() {
            // calling this before self.head.take_next is important to
            // prevent reading freed memory!
            let head_equals_tail = self.is_one_cell();
            let result = self.tail.remove_prev(memory_allocator);

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

    pub fn mem_size_for<A>(length : usize) -> usize {
        mem::size_of::<DoubleLinkedListCell<A>>() * length
    }

    pub fn cell_size<A>() -> usize {
        mem::size_of::<DoubleLinkedListCell<A>>()
    }
}

impl<T> DoubleLinkedList<T> where T : Copy {
    
    /// Deletes current `self.head` from memory and returns copy of its data if it was DoubleLinkedList::Cell.
    /// Returns None otherwise.
    /// # Arguments    
    /// * `memory_allocator` - memory allocator
    pub fn take_head<A>(&mut self, memory_allocator : &mut A) -> Option<T> where A : MemoryAllocator {
        if !self.is_nil() {
            // calling this before self.head.take_next is important to
            // prevent reading freed memory!
            let head_equals_tail = self.is_one_cell();
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
        else {
            None
        }        
    } 
}

/// A type that represents double linked list of cells in memory
#[repr(C)]
pub enum DoubleLinkedListCell<T> {
    /// Type that represents list start point and end points. Used as a marker and doesn't hold any value. 
    Nil,
    /// Represents list cell that holds value of type `T` and has reference to previous and next DoubleLinkedList
    Cell { value : T, prev : heap::SharedBox<DoubleLinkedListCell<T>>, next : heap::SharedBox<DoubleLinkedListCell<T>> }
}

impl<T> DoubleLinkedListCell<T> {

    /// Creates a new cell, which has `prev` and `next` pointing to DoubleLinkedList::Nil.
    /// # Arguments
    /// * `value` - value to put into cell
    /// * `memory_allocator` - memory allocator
    pub fn new<A>(value: T, memory_allocator : &mut A) -> heap::SharedBox<Self> where A : MemoryAllocator {
        let new_cell = DoubleLinkedListCell::Cell {
                value : value,
                next  : heap::SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator),
                prev  : heap::SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator)
        };

        heap::SharedBox::new(new_cell, memory_allocator)        
    }

    /// Creates a new cell, which has `prev` pointing to `self` e.g. previous cell and `next`
    /// pointing to DoubleLinkedList::Nil.
    /// # Arguments
    /// * `value` - value to put into cell
    /// * `memory_allocator` - memory allocator
    pub fn add<A>(&mut self, value: T, memory_allocator : &mut A) -> heap::SharedBox<Self> where A : MemoryAllocator {
        let nil      = heap::SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator);
        let new_cell = DoubleLinkedListCell::Cell {
                value : value,
                next  : nil,
                prev  : heap::SharedBox::from_pointer(self)
        };

        let result = heap::SharedBox::new(new_cell, memory_allocator);

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

    /// Sets `next` to `new_next` if this type is DoubleLinkedList::Cell.
    /// Does nothing otherwise.
    /// # Arguments
    /// * `new_next` - new pointer to next DoubleLinkedList    
    fn set_next(&mut self, new_next : heap::SharedBox<Self>) {
        if let DoubleLinkedListCell::Cell { ref mut next, .. } = *self {
            *next = new_next
        }
    }

    /// Sets `prev` to `new_prev` if this type is DoubleLinkedList::Cell.
    /// Does nothing otherwise.
    /// # Arguments
    /// * `new_prev` - new pointer to previous DoubleLinkedList
    fn set_prev(&mut self, new_prev : heap::SharedBox<Self>) {
        if let DoubleLinkedListCell::Cell { ref mut prev, .. } = *self {
            *prev = new_prev
        }    
    }

    /// Deletes this DoubleLinkedList from memory. Returns `prev` and `next` pointers if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.    
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove<A>(&mut self, memory_allocator : &mut A) -> Option<(heap::SharedBox<Self>, heap::SharedBox<Self>)>
    where A : MemoryAllocator {
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
    pub fn remove_prev<A>(&mut self, memory_allocator : &mut A) -> Option<heap::SharedBox<Self>>
    where A : MemoryAllocator {
        self.remove(memory_allocator).map(|e| e.0)
    }

    /// Deletes this DoubleLinkedList from memory. Returns `next` pointer if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove_next<A>(&mut self, memory_allocator : &mut A) -> Option<heap::SharedBox<Self>>
    where A : MemoryAllocator {
        self.remove(memory_allocator).map(|e| e.1)
    }
}

impl<T> Default for DoubleLinkedListCell<T> {
    fn default() -> Self {
        DoubleLinkedListCell::Nil
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

    /// Tries to return copy of the value in the cell if `self` is DoubleLinkedList::Cell,
    /// or panics if `self` isn't a DoubleLinkedList::Cell.
    pub fn value(&self) -> T {
        self.value_opt().unwrap()
    }
}

impl <T> DoubleLinkedListCell<T> where T : Copy {
    /// Returns copy of the cell data if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take<A>(&self, memory_allocator : &mut A) -> Option<(T, heap::SharedBox<Self>, heap::SharedBox<Self>)> where A : MemoryAllocator {        
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
    pub fn take_prev<A>(&self, memory_allocator : &mut A) -> Option<(T, heap::SharedBox<Self>)> where A : MemoryAllocator {
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
    pub fn take_next<A>(&self, memory_allocator : &mut A) -> Option<(T, heap::SharedBox<Self>)> where A : MemoryAllocator {
        self.take(memory_allocator).map(|e| {
            let (value, _, next) = e;
            (value, next)
        })
    }
}

pub struct DoubleLinkedListIterator<T> {
    current : heap::SharedBox<DoubleLinkedListCell<T>>
}

impl<T> DoubleLinkedListIterator<T> {
    pub fn new(head: heap::SharedBox<DoubleLinkedListCell<T>>) -> DoubleLinkedListIterator<T> {
        DoubleLinkedListIterator { 
            current : head,            
        }
    }
}

impl<T> iter::Iterator for DoubleLinkedListIterator<T> where T : Copy {
    
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match *self.current {
            DoubleLinkedListCell::Cell { value, prev, .. } => {
                self.current = prev;
                Some(value)
            },
            _ => None
        }
    }
}

impl<T> iterator::IteratorExt for DoubleLinkedListIterator<T> where T : Copy { }
