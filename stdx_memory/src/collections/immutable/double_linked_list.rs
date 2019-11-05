use heap;
use MemoryAllocator;
use stdx::iterator;
use core::ops;
use core::ops::Deref;
use core::iter;
use core::ptr;
use core::cmp;
use core::marker;
use stdx::Iterable;

type ListPointer<T, A> = heap::RCBox<DoubleLinkedList<T,A>, A>;
type RCPointer<T, A> = heap::RC<DoubleLinkedList<T, A>, A>;
type StrongListPointer<T, A> = heap::Box<DoubleLinkedList<T,A>, A>;

#[repr(C, packed)]
pub struct DoubleLinkedList<T, A> where A : MemoryAllocator {
    value : T,
    prev : Option<RCPointer<T, A>>,
    next : Option<RCPointer<T, A>>,
    phantom : marker::PhantomData<A>,
}

impl<T, A> DoubleLinkedList<T, A> where A : MemoryAllocator {

    pub fn prev(&self) -> Option<RCPointer<T, A>> {
        self.prev.as_ref().map(|rc| heap::RC::clone(rc))
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn next(&self) -> Option<RCPointer<T, A>> {
        self.next.as_ref().map(|rc| heap::RC::clone(rc))
    }

    pub fn value_mut(&mut self) -> &mut T { &mut self.value}

    fn new_cell(value: T) -> Self {
        DoubleLinkedList {
            value,
            next  : None,
            prev  : None,
            phantom : marker::PhantomData
        }
    }

    pub fn new(value: T, memory_allocator : &mut A) -> StrongListPointer<T, A>  {  heap::Box::new(DoubleLinkedList::new_cell(value), memory_allocator)  }

    pub fn new_rc(value: T, memory_allocator : &mut A) -> RCPointer<T, A> {  heap::RC::new(DoubleLinkedList::new_cell(value), memory_allocator)  }

    pub fn add(arg : &mut RCPointer<T, A>, value: T, memory_allocator : &mut A) -> RCPointer<T, A> {

        let new_cell = DoubleLinkedList {
                value,
                next  : None,
                prev  : Some(heap::RC::clone(arg)),
                phantom : marker::PhantomData
        };

        let result = heap::RC::new(new_cell, memory_allocator);

        arg.set_next(heap::RC::clone(&result));

        heap::RC::clone(&result)
    }

    pub fn set_next(&mut self, arg : RCPointer<T, A>) {
        self.next = Some(arg)
    }

    /// Deletes this DoubleLinkedList from memory. Returns `prev` and `next` pointers if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.    
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    /*pub fn remove(mut self) -> (Option<ListPointer<T, A>>, Option<ListPointer<T, A>>) {
        if let Some(mut next) = self.next.as_mut() {

            next.prev.take();
            next.prev = Some(heap::WeakBox::from_pointer(&self.prev.take().unwrap()));
        }

        if let Some(mut prev) = self.prev.as_mut() {

            let b = prev.next.take();
            let bv = b;
            prev.next = Some(heap::WeakBox::from_pointer(&self.next.take().unwrap()));
        }

        let result =(self.prev, self.next);

        result
    }*/

    pub fn modify_neighbour_connections(mut a : RCPointer<T,A>)  {
        let prev_addr = a.prev();
        let next_addr = a.next();

        if let Some(mut next) = a.next.as_mut() {

            next.prev.take();
            next.prev = prev_addr;
        }

        if let Some(mut prev) = a.prev.as_mut() {

            prev.next.take();
            prev.next = next_addr
        }
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


    
}

/*impl<T, M> Iterable for DoubleLinkedList<T, M> where M : MemoryAllocator {

    type Item = T;

    type IntoIter = DoubleLinkedListIterator<T, M>;

    fn iterator(&self) -> DoubleLinkedListIterator<T, M> {
        DoubleLinkedListIterator::new()
    }
}*/

impl<T, A> cmp::Ord for DoubleLinkedList<T, A> where T : cmp::Ord, A : MemoryAllocator {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.value().cmp(other.value())
    }
}

impl<T, A> cmp::PartialOrd for DoubleLinkedList<T, A> where T : cmp::PartialOrd, A : MemoryAllocator {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value().partial_cmp(other.value())
    }
}

impl<T, A> cmp::Eq for DoubleLinkedList<T, A> where T : cmp::Eq, A : MemoryAllocator {

}

impl<T, A> cmp::PartialEq for DoubleLinkedList<T, A> where T : cmp::PartialEq, A : MemoryAllocator {
    fn eq(&self, other: &Self) -> bool {
        self.value().eq(other.value())
    }
}

pub struct DoubleLinkedListIterator<T, M> where M : MemoryAllocator {
    current : Option<RCPointer<T, M>>
}

impl<T, M> DoubleLinkedListIterator<T, M> where M : MemoryAllocator {
    pub fn new(head: Option<RCPointer<T, M>>) -> DoubleLinkedListIterator<T, M> {
        DoubleLinkedListIterator {
            current : head
        }
    }
}

impl<T, M> iter::Iterator for DoubleLinkedListIterator<T, M> where M : MemoryAllocator {

    type Item = RCPointer<T, M>;

    fn next(&mut self) -> Option<RCPointer<T, M>> {
        match self.current.take() {
            Some(cell) => {
                    self.current = cell.next();
                    Some(cell)
                },
            _ => None
            }
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