use heap;
use MemoryAllocator;
use stdx::iterator;
use core::ops;
use core::ops::Deref;
use core::iter;
/*
#[repr(C, packed)]
pub struct DoubleLinkedList<T> {
    value : T,
    prev : Option<heap::Box<DoubleLinkedList<T>>>,
    next : Option<heap::Box<DoubleLinkedList<T>>>
}

impl<T> DoubleLinkedList<T> {

    pub fn value(self) -> T {
        self.value
    }

    pub fn new<A>(value: T, memory_allocator : &mut A) -> heap::Box<Self> where A : MemoryAllocator  {
        let new_cell = DoubleLinkedList {
                value : value,
                next  : None,
                prev  : None
        };

        heap::Box::new(new_cell, memory_allocator)
    }

    pub fn add<A>(&mut self, value: T, memory_allocator : &mut A) -> heap::Box<Self>  where A : MemoryAllocator {
        let new_cell = DoubleLinkedList {
                value : value,
                next  : None,
                prev  : Some(heap::Box::from_pointer(self))
        };

        let result = heap::Box::new(new_cell, memory_allocator);
        let result_copy = heap::Box::from_pointer(result.deref());

        self.next = Some(result_copy);
        result
    }

    /// Deletes this DoubleLinkedList from memory. Returns `prev` and `next` pointers if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.    
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove(self) -> (Option<heap::Box<Self>>, Option<heap::Box<Self>>) {
        let result = self.take();
        (result.1, result.2)
    }

    /// Deletes this DoubleLinkedList from memory. Returns `prev` pointer if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove_prev(self) -> Option<heap::Box<Self>> {
        self.remove().0
    }

    /// Deletes this DoubleLinkedList from memory. Returns `next` pointer if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove_next(self) -> Option<heap::Box<Self>>
     {
        self.remove().1
    }    

    pub fn single_cell(&self) -> bool {
        self.prev.is_none() && self.next.is_none()
    }

    /// Determines if this type is DoubleLinkedList::Cell which has `prev` pointing to DoubleLinkedList::Nil
    pub fn is_start(&self) -> bool {
        self.prev.is_none()
    }

    /// Determines if this type is DoubleLinkedList::Cell which has `next` pointing to DoubleLinkedList::Nil
    pub fn is_end(&self) -> bool {
        self.next.is_none()
    }

    /// Returns copy of the cell data if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take(self) -> (T, Option<heap::Box<Self>>, Option<heap::Box<Self>>)  {

        if let Some(mut next) = self.next {
            //let prev_copy = heap::Box::from_pointer(self.prev.deref());
            next.prev = self.prev;
        }

        if let Some(mut prev) = self.prev {
            prev.next = self.next;
        }                 

        let result = (self.value, self.prev, self.next);

        result
    }

    /// Returns copy of the cell data and pointer to previous DoubleLinkedList
    /// if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take_prev(self) -> (T, Option<heap::Box<Self>>)  {
        let (value, prev, _) = self.take();
        (value, prev)
    }

    /// Returns copy of the cell data and pointer to next DoubleLinkedList
    /// if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take_next(self) -> (T, Option<heap::Box<Self>>)  {
        let (value, _, next) = self.take();
        (value, next)    
    }
}

impl<T> ops::Deref for DoubleLinkedList<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

pub struct DoubleLinkedListIterator<T> {
    current : Option<heap::Box<DoubleLinkedList<T>>>
}

impl<T> DoubleLinkedListIterator<T> {
    fn new(head: Option<heap::Box<DoubleLinkedList<T>>>) -> DoubleLinkedListIterator<T> {
        DoubleLinkedListIterator { 
            current : head,
        }
    }
}

impl<T> iter::Iterator for DoubleLinkedListIterator<T> {
    
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.current.take() {
            Some(cell) => {
                let result = cell.take_prev();
                self.current = result.1;
                Some(result.0)
            },
            _ => None
        }
    }
}

impl<T> iterator::IteratorExt for DoubleLinkedListIterator<T>{ }
*/