use core::fmt;
use core::iter;
use core::marker;
use MemoryAllocator;
use heap::SharedBox;

/// Type that represents linked list of cells.
#[repr(C)]
pub enum LinkedList<T> {
    /// Represents list starting point. Used as a marker that doesn't hold any value
    Nil,
    /// Represents list cell that holds value of type `T` and has reference to previous LinkedList
    Cell { value: T, prev: SharedBox<LinkedList<T>> },
}

impl<T> LinkedList<T> {

    /// Creates a new cell, which has `prev` pointing to LinkedList::Nil
    /// # Arguments
    /// * `value` - value to put into cell
    /// * `memory_allocator` - memory allocator
    pub fn new<A>(value: T, memory_allocator : &mut A) -> SharedBox<LinkedList<T>> where A : MemoryAllocator {
        let result = LinkedList::Cell {
                value: value,
                prev: SharedBox::new(LinkedList::Nil, memory_allocator),
        };

        SharedBox::new(result, memory_allocator)
    }

    /// Creates a new cell, which has `prev` pointing to `self` e.g. previous cell
    /// # Arguments
    /// * `value` - value to put into cell
    /// * `memory_allocator` - memory allocator
    pub fn add<A>(&self, value: T, memory_allocator : &mut A) -> SharedBox<LinkedList<T>> where A : MemoryAllocator {
        let result = LinkedList::Cell {
                value : value,
                prev  : SharedBox::from_pointer(self),
        };

        SharedBox::new(result, memory_allocator)
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
        
    /// Returns copy of the cell data if `self` is cell then clears memory associated with that cell,
    /// does nothing if `self` is LinkedList::Nil
    /// # Arguments    
    /// * `memory_allocator` - memory allocator
    pub fn take<A>(&self, memory_allocator : &mut A) -> Option<(T, SharedBox<LinkedList<T>>)>
    where A : MemoryAllocator {
        match *self {
            LinkedList::Cell { value, prev } => { 
                let result = Some((value, prev));
                memory_allocator.free(self as *const _ as usize);
                result        
            },
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

pub struct LinkedListIterator<T> where T : Copy {
    current: SharedBox<LinkedList<T>>,
    phantom : marker::PhantomData<T>
}

impl<T> LinkedListIterator<T> where T : Copy {
    pub fn new(head: SharedBox<LinkedList<T>>) -> LinkedListIterator<T> {
        LinkedListIterator { 
            current : head,
            phantom : marker::PhantomData 
        }
    }
}

impl<T> iter::Iterator for LinkedListIterator<T> where T : Copy {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match *self.current {
            LinkedList::Cell { value, prev } => {
                self.current = prev;
                Some(value)
                
            },
            _ => None
        }
    }
}