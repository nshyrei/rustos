use heap;
use MemoryAllocator;
use stdx::iterator;
use core::ops;
use core::ops::Deref;
use core::iter;
use core::ptr;
use core::cmp;

type ListPointer<T> = heap::WeakBox<DoubleLinkedList<T>>;
type StrongListPointer<T, A> = heap::Box<DoubleLinkedList<T>, A>;


#[repr(C, packed)]
pub struct DoubleLinkedList<T> {
    value : T,
    prev : Option<ListPointer<T>>,
    next : Option<ListPointer<T>>
}

impl<T> DoubleLinkedList<T> {

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn new<A>(value: T, memory_allocator : &mut A) -> StrongListPointer<T, A> where A : MemoryAllocator {
        let new_cell = DoubleLinkedList {
                value,
                next  : None,
                prev  : None
        };

        heap::Box::new(new_cell, memory_allocator)
    }

    pub fn add<A>(&mut self, value: T, memory_allocator : &mut A) -> StrongListPointer<T, A> where A : MemoryAllocator {

        let new_cell = DoubleLinkedList {
                value,
                next  : None,
                prev  : Some(heap::WeakBox::from_pointer(self))
        };

        heap::Box::new(new_cell, memory_allocator)
    }
    
    pub fn next_mut(&mut self) -> &mut Option<ListPointer<T>> {
        &mut self.next
    }

    /// Deletes this DoubleLinkedList from memory. Returns `prev` and `next` pointers if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.    
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove(mut self) -> (Option<ListPointer<T>>, Option<ListPointer<T>>) {
        let result = self.take();
        (result.1, result.2)
    }

    /// Deletes this DoubleLinkedList from memory. Returns `prev` pointer if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove_prev(mut self) -> Option<ListPointer<T>> {
        self.remove().0
    }

    /// Deletes this DoubleLinkedList from memory. Returns `next` pointer if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove_next(mut self) -> Option<ListPointer<T>>
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
    pub fn take(mut self) -> (T, Option<ListPointer<T>>, Option<ListPointer<T>>)  {

        if let Some(mut next) = self.next.as_mut() {

            if let Some(prev) = self.prev.as_ref() {
                next.prev.take();
                next.prev = Some(heap::WeakBox::from_pointer(prev.deref()));
            }
        }

        if let Some(mut prev) = self.prev.as_mut() {

            if let Some(next) = self.next.as_ref() {
                prev.next.take();
                prev.next = Some(heap::WeakBox::from_pointer(next.deref()));
            }

        }

        (self.value, self.prev, self.next)
    }
    
    /// Returns copy of the cell data and pointer to previous DoubleLinkedList
    /// if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take_prev(mut self) -> (T, Option<ListPointer<T>>)  {
        let (value, prev, _) = self.take();
        (value, prev)
    }

    /// Returns copy of the cell data and pointer to next DoubleLinkedList
    /// if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take_next(mut self) -> (T, Option<ListPointer<T>>)  {
        let (value, _, next) = self.take();
        (value, next)    
    }
    
}

impl<T> cmp::Ord for DoubleLinkedList<T> where T : cmp::Ord {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.value().cmp(other.value())
    }
}

impl<T> cmp::PartialOrd for DoubleLinkedList<T> where T : cmp::PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value().partial_cmp(other.value())
    }
}

impl<T> cmp::Eq for DoubleLinkedList<T> where T : cmp::Eq {

}

impl<T> cmp::PartialEq for DoubleLinkedList<T> where T : cmp::PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.value().eq(other.value())
    }
}

/*
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