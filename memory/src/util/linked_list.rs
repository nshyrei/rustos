use core::fmt;
use core::iter;
use core::marker;
use allocator::MemoryAllocator;
use util::bump_allocator::BumpAllocator;
use util::SharedBox;

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
    pub fn new(value: T, memory_allocator : &mut BumpAllocator) -> SharedBox<LinkedList<T>> {
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
    pub fn add(&self, value: T, memory_allocator : &mut BumpAllocator) -> SharedBox<LinkedList<T>> {
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

    // # Comment out reason : unused, but can be usefull in the future
    /*
    /// Returns reference to cell data if `self` is LinkedList::Cell, returns None if `self` is LinkedList::Nil
    pub fn value_ref(&self) -> Option<&T> {
        match *self {
            LinkedList::Cell { ref value, .. } => Some(value),
            _ => None
        }
    }

    /// Returns mut reference to cell data if `self` is LinkedList::Cell, returns None if `self` is LinkedList::Nil
    pub fn value_mut_ref(&mut self) -> Option<&mut T> {
        match *self {
            LinkedList::Cell { ref mut value, .. } => Some(value),
            _ => None
        }
    }

    /// Returns previous LinkedList if `self` is LinkedList::Cell, returns None if `self` is LinkedList::Nil    
    pub fn prev(&self) -> Option<&SharedBox<LinkedList<T>>> {
        match *self {
            LinkedList::Cell { ref prev, .. } => Some(prev),
            _ => None
        }
    }
    */   
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

    // # Comment out reason : unused, but can be usefull in the future
    /*
    /// Returns copy of the cell data if `self` is cell then clears memory associated with that cell,
    /// does nothing and returns None if `self` is LinkedList::Nil
    /// otherwise returns None
    pub fn take(self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<LinkedList<T>>)> {
        let result = (self.value(), self.next);
        memory_allocator.free(mem::size_of::<LinkedList<T>>());
        result
    }
    */
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