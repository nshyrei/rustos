use stdx::ptr;
use core::mem;
use core::ptr::write_unaligned;
use util::bump_allocator::BumpAllocator;
use allocator::MemoryAllocator;
use util::Box;

#[repr(C)]
pub enum DoubleLinkedList<T> {
    Nil,
    Cons { value : T, prev : Box<DoubleLinkedList<T>>, next : Box<DoubleLinkedList<T>> }
}


impl<T> DoubleLinkedList<T> {

    pub fn add(&self, value: T, memory_allocator : &mut BumpAllocator) -> Box<Self> {
        let nil = Box::new(DoubleLinkedList::Nil, memory_allocator);
        let result = DoubleLinkedList::Cons {
                value : value,
                next  : nil,
                prev  : Box::from_pointer(self)
        };

        Box::new(result, memory_allocator)        
    }

    pub fn value_ref_opt(&self) -> Option<&T> {
        match self {
            &DoubleLinkedList::Cons { ref value, .. } => Some(value),
            &DoubleLinkedList::Nil => None
        }
    }

    pub fn value_ref(&self) -> &T {
        DoubleLinkedList::value_ref_opt(&self).expect("Trying to get value From Nil")
    }

    pub fn next_opt(&self) -> Option<&DoubleLinkedList<T>> {
        match self {
            &DoubleLinkedList::Cons { ref next, .. } => Some(next.pointer()),
            &DoubleLinkedList::Nil => None
        }
    }

    pub fn prev_opt(&self) -> Option<&DoubleLinkedList<T>> {
        match self {
            &DoubleLinkedList::Cons { ref prev, .. } => Some(prev.pointer()),
            &DoubleLinkedList::Nil => None
        }
    }

    pub fn set_next(&mut self, new_next : Box<DoubleLinkedList<T>>) {
        if let &mut DoubleLinkedList::Cons { ref mut next, .. } = self {
            *next = new_next
        }
    }

    pub fn set_prev(&mut self, new_prev : Box<DoubleLinkedList<T>>) {
        if let &mut DoubleLinkedList::Cons { ref mut prev, .. } = self {
            *prev = new_prev
        }        
    }
}

impl <T> DoubleLinkedList<T> where T : Copy {
    pub fn take_opt(self, memory_allocator : &mut BumpAllocator) -> Option<T> {
        match self {
            DoubleLinkedList::Cons { value, mut prev, mut next } => {
                next.set_prev(Box::from_pointer(prev.pointer()));
                prev.set_next(Box::from_pointer(next.pointer()));
                
                memory_allocator.free(mem::size_of::<Self>());
                
                Some(value)
            },
            DoubleLinkedList::Nil => None
        }        
    }
}