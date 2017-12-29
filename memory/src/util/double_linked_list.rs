use core::mem;
use util::bump_allocator::BumpAllocator;
use allocator::MemoryAllocator;
use util::{Box, SharedBox};
use util::array::Array;
use frame::Frame;


pub struct BuddyFreeList {
    frame_to_free_buddy : Array<SharedBox<DoubleLinkedListCell<usize>>>,
    free_blocks         : DoubleLinkedList<usize>
}

impl BuddyFreeList {
    pub fn new(frame_count : usize, memory_allocator : &mut BumpAllocator) -> Self {
        
        BuddyFreeList {
            frame_to_free_buddy : Array::new(frame_count, memory_allocator),
            free_blocks         : DoubleLinkedList::new(memory_allocator),            
        }
    }

    pub fn is_free(&self, block_start_address : usize) -> bool {
        !self.is_in_use(block_start_address)
    }

    pub fn is_in_use(&self, block_start_address : usize) -> bool {
        // todo block_start_address or frame number will be out of range
        self.frame_to_free_buddy.elem_ref(block_start_address).is_cell()
    }

    pub fn set_in_use(&mut self, block_start_address : usize, memory_allocator : &mut BumpAllocator) {
        if self.is_free(block_start_address) {
            let cell = self.frame_to_free_buddy.value(block_start_address);
            self.free_blocks.remove(cell, memory_allocator);
            self.frame_to_free_buddy.update(block_start_address, SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator));        
        }
    }

    pub fn set_free(&mut self, block_start_address : usize, memory_allocator : &mut BumpAllocator) {
        if self.is_in_use(block_start_address) {
            let cell = self.free_blocks.add_to_tail(block_start_address, memory_allocator);            
            self.frame_to_free_buddy.update(block_start_address, cell);        
        }
    }

    pub fn first_free_block(&mut self, memory_allocator : &mut BumpAllocator) -> Option<usize> {
        let result = self.free_blocks.take_head(memory_allocator);

        if let Some(index) = result {
            self.frame_to_free_buddy.update(index, SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator));
        }

        result
    }
}

pub struct DoubleLinkedList<T> {    
    head : SharedBox<DoubleLinkedListCell<T>>,
    tail : SharedBox<DoubleLinkedListCell<T>>
}

impl<T> DoubleLinkedList<T> {
    pub fn new(memory_allocator : &mut BumpAllocator) -> Self {
        DoubleLinkedList {            
            head : SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator),
            tail : SharedBox::new(DoubleLinkedListCell::Nil, memory_allocator)
        }
    }
    
    pub fn add_to_tail(&mut self, value : T, memory_allocator : &mut BumpAllocator) -> SharedBox<DoubleLinkedListCell<T>> {
        let new_cell = self.tail.add(value, memory_allocator);

        self.tail = new_cell;

        if self.head.is_nil() {
            self.head = new_cell;
        }

        new_cell
    }
    
    pub fn remove(&mut self, cell : SharedBox<DoubleLinkedListCell<T>>, memory_allocator : &mut BumpAllocator) {
        if self.head_equals_tail() && cell.pointer_equal(&self.head) {
            let nil = self.head.remove_next(memory_allocator).unwrap();
            self.head = nil;
            self.tail = nil;
        }
        else if cell.pointer_equal(&self.head) {
            let new_head = self.head.remove_next(memory_allocator).unwrap();
            self.head = new_head;
        }
        else if cell.pointer_equal(&self.tail) {
            let new_tail = self.tail.remove_prev(memory_allocator).unwrap();
            self.tail = new_tail;                
        }
        else {
            cell.pointer_mut().remove(memory_allocator);
        }
    }

    fn head_equals_tail(&self) -> bool {
        (self.head.is_nil() && self.tail.is_nil()) || self.head.pointer_equal(&self.tail)
    }    
}

impl<T> DoubleLinkedList<T> where T : Copy {
    
    pub fn take_head(&mut self, memory_allocator : &mut BumpAllocator) -> Option<T> {
        let result = self.head.take_next(memory_allocator);

        if let Some((_, new_head)) = result {
            if self.head_equals_tail() {
                self.head = new_head;
                self.tail = new_head;
            }
            else {
                self.head = new_head;
            }
        }

        result.map(|e| e.0)
    }    
}

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
        match *self {
            DoubleLinkedListCell::Cell { .. } => true,
            _ => false
        }
    }

    pub fn is_start(&self) -> bool {
        match *self {
            DoubleLinkedListCell::Cell { prev, .. } => prev.is_nil(),
            _ => false
        }
    }

    pub fn is_end(&self) -> bool {
        match *self {
            DoubleLinkedListCell::Cell { next, .. } => next.is_nil(),
            _ => false
        }
    }

    pub fn is_nil(&self) -> bool {
        !self.is_cell()
    }

    pub fn value_ref_opt(&self) -> Option<&T> {
        match *self {
            DoubleLinkedListCell::Cell { ref value, .. } => Some(value),
            _ => None
        }
    }

    pub fn value_ref(&self) -> &T {
        DoubleLinkedListCell::value_ref_opt(&self).expect("Trying to get value From Nil")
    }

    pub fn next_opt(&self) -> Option<&Self> {
        match *self {
            DoubleLinkedListCell::Cell { ref next, .. } => Some(next.pointer()),
            _ => None
        }
    }

    pub fn prev_opt(&self) -> Option<&Self> {
        match *self {
            DoubleLinkedListCell::Cell { ref prev, .. } => Some(prev.pointer()),
            _ => None
        }
    }

    pub fn set_next(&mut self, new_next : SharedBox<Self>) {
        if let DoubleLinkedListCell::Cell { ref mut next, .. } = *self {
            *next = new_next
        }
    }

    pub fn set_prev(&mut self, new_prev : SharedBox<Self>) {
        if let DoubleLinkedListCell::Cell { ref mut prev, .. } = *self {
            *prev = new_prev
        }    
    }

    pub fn remove(&mut self, memory_allocator : &mut BumpAllocator) -> Option<(SharedBox<Self>, SharedBox<Self>)> {
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

    pub fn remove_prev(&mut self, memory_allocator : &mut BumpAllocator) -> Option<SharedBox<Self>> {
        self.remove(memory_allocator).map(|e| e.0)
    }

    pub fn remove_next(&mut self, memory_allocator : &mut BumpAllocator) -> Option<SharedBox<Self>> {
        self.remove(memory_allocator).map(|e| e.1)
    }
}

impl<T> DoubleLinkedListCell<T> where T : Copy {

    pub fn value_opt(&self) -> Option<T> {
        match *self {
            DoubleLinkedListCell::Cell { value, .. } => Some(value),
            _ => None
        }
    }
}

impl <T> DoubleLinkedListCell<T> where T : Copy {
    pub fn take(&self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<Self>, SharedBox<Self>)> {        
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

    pub fn take_prev(&self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<Self>)> {
        self.take(memory_allocator).map(|e| {
            let (value, prev, _) = e;
            (value, prev)
        })
    }

    pub fn take_next(&self, memory_allocator : &mut BumpAllocator) -> Option<(T, SharedBox<Self>)> {
        self.take(memory_allocator).map(|e| {
            let (value, _, next) = e;
            (value, next)
        })
    }
}