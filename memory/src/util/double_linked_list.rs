use core::mem;
use util::bump_allocator::BumpAllocator;
use allocator::MemoryAllocator;
use util::{Box, SharedBox};
use util::array::Array;
use frame::Frame;


pub struct BuddyFreeList {
    frame_to_free_buddy : Array<SharedBox<DoubleLinkedListCell<usize>>>,
    head : Box<DoubleLinkedListCell<usize>>
}

impl BuddyFreeList {
    pub fn new(frame_count : usize, memory_allocator : &mut BumpAllocator) -> Self {
        
        BuddyFreeList {
            frame_to_free_buddy : Array::new(frame_count, memory_allocator),
            head : Box::new(DoubleLinkedListCell::Nil, memory_allocator)
        }
    }
                
    pub fn add(&mut self, block_start_address : usize, memory_allocator : &mut BumpAllocator) {
        let new_cell = self.head.add(block_start_address, memory_allocator);        

        let frame_number = Frame::from_address(block_start_address).number();

        self.frame_to_free_buddy.update(frame_number, new_cell)        
    }
}
/*
impl <T> BuddyFreeList<T> where T : Copy {

    pub fn first_free(&mut self, memory_allocator : &mut BumpAllocator) -> Option<T> {
        self.head.take_opt(memory_allocator).map(|e| {
            let (value, prev, next) = e;
            value
        })
    }
}
*/
pub struct DoubleLinkedList<T> {
    head : Box<DoubleLinkedListCell<T>>
}

impl<T> DoubleLinkedList<T> {
    pub fn new(memory_allocator : &mut BumpAllocator) -> Self {

        DoubleLinkedList {            
            head : Box::new(DoubleLinkedListCell::Nil, memory_allocator)
        }
    }

    pub fn has_head(&self) -> bool {
        self.head.is_cell()
    }
}

/*
impl<T> DoubleLinkedList<T> where T : Copy {
    pub fn take_head(&mut self, memory_allocator : &mut BumpAllocator) -> Option<T> {
        let result = match self.head.pointer_mut() {
            &mut DoubleLinkedListCell::Cell { value, ref mut prev, ref mut next } => {
                next.set_prev(prev);
                prev.set_next(next);                
                
                Some((value, next))
            },
            &mut DoubleLinkedListCell::Nil => None
        };

        result.map(|e| {            
            memory_allocator.free(self.head.pointer() as *const _ as usize);

            let (result, next) = e;
            self.head = next;
            result
        })
    }
}
*/

#[repr(C)]
pub enum DoubleLinkedListCell<T> {
    Nil,
    Cell { value : T, prev : SharedBox<DoubleLinkedListCell<T>>, next : SharedBox<DoubleLinkedListCell<T>> }
}


impl<T> DoubleLinkedListCell<T> {

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

    pub fn is_cell(&self) -> bool {
        match self {
            &DoubleLinkedListCell::Cell { .. } => true,
            _ => false
        }
    }

    pub fn value_ref_opt(&self) -> Option<&T> {
        match self {
            &DoubleLinkedListCell::Cell { ref value, .. } => Some(value),
            _ => None
        }
    }

    pub fn value_ref(&self) -> &T {
        DoubleLinkedListCell::value_ref_opt(&self).expect("Trying to get value From Nil")
    }

    pub fn next_opt(&self) -> Option<&Self> {
        match self {
            &DoubleLinkedListCell::Cell { ref next, .. } => Some(next.pointer()),
            _ => None
        }
    }

    pub fn prev_opt(&self) -> Option<&Self> {
        match self {
            &DoubleLinkedListCell::Cell { ref prev, .. } => Some(prev.pointer()),
            _ => None
        }
    }

    pub fn set_next(&mut self, new_next : SharedBox<Self>) {
        if let &mut DoubleLinkedListCell::Cell { ref mut next, .. } = self {
            *next = new_next
        }
    }

    pub fn set_prev(&mut self, new_prev : SharedBox<Self>) {
        if let &mut DoubleLinkedListCell::Cell { ref mut prev, .. } = self {
            *prev = new_prev
        }    
    }
}

impl <T> DoubleLinkedListCell<T> where T : Copy {
    pub fn take(self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<Self>, SharedBox<Self>)> {
        let self_address = &self as *const _ as usize;
        let result = match self {
            DoubleLinkedListCell::Cell { value, mut prev, mut next } => {
                next.set_prev(prev);
                prev.set_next(next);                

                let result = (value, prev, next);
                Some(result)
            },
            _ => None
        };

        memory_allocator.free(self_address);
        result
    }

    pub fn take_prev(self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<Self>)> {
        self.take(memory_allocator).map(|e| {
            let (value, prev, _) = e;
            (value, prev)
        })
    }

    pub fn take_next(self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<Self>)> {
        self.take(memory_allocator).map(|e| {
            let (value, _, next) = e;
            (value, next)
        })
    }
}