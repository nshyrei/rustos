use core::iter;
use core::mem;
use core::fmt;
use core::ptr;
use allocator::MemoryAllocator;
use util::bump_allocator::BumpAllocator;
use stdx::smart_ptr;
use util::SharedBox;
use core;

/*
    A classic cons list
*/

#[repr(C)]
pub enum LinkedList<T> {
    Nil,
    Cell { value: T, prev: SharedBox<LinkedList<T>> },
}

impl<T> LinkedList<T> {

    pub fn size() -> usize {
        mem::size_of::<T>() + mem::size_of::<Option<SharedBox<LinkedList<T>>>>()    
    }

    pub fn new(value: T, memory_allocator : &mut BumpAllocator) -> SharedBox<LinkedList<T>> {
        let result = LinkedList::Cell {
                value: value,
                prev: SharedBox::new(LinkedList::Nil, memory_allocator),
        };

        SharedBox::new(result, memory_allocator)
    }

    pub fn value_ref(&self) -> Option<&T> {
        match *self {
            LinkedList::Cell { ref value, .. } => Some(value),
            _ => None
        }
    }

    pub fn prev(&self) -> Option<&SharedBox<LinkedList<T>>> {
        match *self {
            LinkedList::Cell { ref prev, .. } => Some(prev),
            _ => None
        }
    }

    pub fn add(&self, value: T, memory_allocator : &mut BumpAllocator) -> SharedBox<LinkedList<T>> {
        let result = LinkedList::Cell {
                value : value,
                prev  : SharedBox::from_pointer(self),
        };

        SharedBox::new(result, memory_allocator)        
    }    
}

impl <T> LinkedList<T> where T : Copy {
    pub fn value(&self) -> Option<T> {
        match *self {
            LinkedList::Cell { value, .. } => Some(value),
            _ => None
        }
    }

    /*
    pub fn take(self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<LinkedList<T>>)> {
        let result = (self.value(), self.next);
        memory_allocator.free(mem::size_of::<LinkedList<T>>());
        result
    }
    */
}

impl<T> fmt::Display for LinkedList<T> where T : Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"")
    }
}

/*
pub struct LinkedListIterator<T> {
    current: Option<SharedBox<LinkedList<T>>>,
}

impl<T> LinkedListIterator<T> {
    pub fn new(head: SharedBox<LinkedList<T>>) -> LinkedListIterator<T> {
        LinkedListIterator { current: Some(head) }
    }
}

impl<T> iter::Iterator for LinkedListIterator<T> {
    type Item = SharedBox<T>;

    fn next(&mut self) -> Option<SharedBox<T>> {
        match self.current {
            Some(li) => {
                let current = li.pointer();
                self.current = current.next();
                Some(current.value_ref())
            }
            None => None,
        }
    }
}
*/