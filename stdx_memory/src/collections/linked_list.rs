use core::fmt;
use core::iter;
use core::marker;
use core::mem;
use MemoryAllocator;
use heap::SharedBox;
use heap::Box;
use heap::WeakBox;

/// Type that represents linked list of cells.
#[repr(C)]
pub enum LinkedList<T> {
    /// Represents list starting point. Used as a marker that doesn't hold any value
    Nil,
    /// Represents list cell that holds value of type `T` and has reference to previous LinkedList
    Cell { value: T, prev: WeakBox<LinkedList<T>> },
}

impl<T> LinkedList<T> {

    /// Creates a new cell, which has `prev` pointing to LinkedList::Nil
    /// # Arguments
    /// * `value` - value to put into cell
    /// * `memory_allocator` - memory allocator
    pub fn new<A>(value: T, memory_allocator : &mut A) -> Box<Self, A> where A : MemoryAllocator {
        let result = LinkedList::Cell {
                value: value,
                prev: WeakBox::new(LinkedList::Nil,memory_allocator),
        };

        Box::new(result, memory_allocator)
    }

    pub fn nil<A>(memory_allocator : &mut A) -> Box<Self, A> where A : MemoryAllocator  {
        Box::new(LinkedList::Nil, memory_allocator)
    }

    /// Creates a new cell, which has `prev` pointing to `self` e.g. previous cell
    /// # Arguments
    /// * `value` - value to put into cell
    /// * `memory_allocator` - memory allocator
    pub fn add<A>(&self, value: T, memory_allocator : &mut A) -> Box<Self, A> where A : MemoryAllocator {
        let result = LinkedList::Cell {
                value : value,
                prev  : WeakBox::from_pointer(self),
        };

        Box::new(result, memory_allocator)
    }

    /// Determines if this LinkedList type is LinkedList::Nil    
    pub fn is_nil(&self) -> bool {
        !self.is_cell()
    }

    /// Determines if this LinkedList type is LinkedList::Cell
    pub fn is_cell(&self) -> bool {
        match *self {
            LinkedList::Cell { .. } => true,
            _ => false
        }
    }

    /// Returns copy of the cell data if `self` is cell then clears memory associated with that cell,
    /// does nothing if `self` is LinkedList::Nil
    /// # Arguments    
    /// * `memory_allocator` - memory allocator
    pub fn take(&mut self) -> Option<(T, WeakBox<Self>)> {
        match mem::replace(self, LinkedList::Nil) {
            LinkedList::Cell { value, prev } => Some((value, prev)),
            _ => None
        }        
    }
}

impl <T> LinkedList<T> where T : Copy {
    /// Returns copy of the value in the cell if `self` is LinkedList::Cell,
    /// otherwise returns None
    pub fn value(&self) -> Option<T> {
        match *self {
            LinkedList::Cell { value, .. } => Some(value),
            _ => None
        }
    }            
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        LinkedList::Nil
    }
}

impl<T> fmt::Display for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"")
    }
}
/*
pub struct LinkedListIterator<T> where T : Copy {
    current: Box<LinkedList<T>>,    
}

impl<T> LinkedListIterator<T> where T : Copy {
    pub fn new(head: Box<LinkedList<T>>) -> LinkedListIterator<T> {
        LinkedListIterator { 
            current : head,            
        }
    }
}

impl<T> iter::Iterator for LinkedListIterator<T> where T : Copy {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.current.take() {
            Some((value, prev)) => {
                self.current = prev;
                Some(value)
            },
            _ => None
        }
        
    }
}
*/