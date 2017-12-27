use core::iter;
use core::mem;
use core::fmt;
use core::ptr;
use allocator::MemoryAllocator;
use util::bump_allocator::BumpAllocator;
use stdx::smart_ptr;
use util::Box;
use util::SharedBox;
use core;

/*
    A classic cons list
*/

#[repr(C)]
//#[derive(Copy, Clone)]
pub struct FreeList<T> {
    value: T,
    next: Option<SharedBox<FreeList<T>>>,
}

impl<T> FreeList<T> {

    pub fn size() -> usize {
        mem::size_of::<T>() + mem::size_of::<Option<SharedBox<FreeList<T>>>>()    
    }

    pub fn new(value: T, memory_allocator : &mut BumpAllocator) -> SharedBox<FreeList<T>> {
        let result = FreeList {
                value: value,
                next: None,
        };

        SharedBox::new(result, memory_allocator)
    }

    pub fn value_ref(&self) -> &T {
        &self.value
    }

    pub fn next(&self) -> Option<SharedBox<FreeList<T>>> {
        self.next
    }

    pub fn add(&self, value: T, memory_allocator : &mut BumpAllocator) -> SharedBox<FreeList<T>> {
        let result = FreeList {
                value : value,
                next  : Some(SharedBox::from_pointer(self)),
        };

        SharedBox::new(result, memory_allocator)        
    }

    pub fn free(self, memory_allocator : &mut BumpAllocator) -> Option<SharedBox<FreeList<T>>> {
        memory_allocator.free(mem::size_of::<FreeList<T>>());
            
        self.next
    }
}

impl <T> FreeList<T> where T : Copy {
    pub fn value(&self) -> T {
        self.value
    }

    pub fn take(self, memory_allocator : &mut BumpAllocator) -> (T, Option<SharedBox<FreeList<T>>>) {
        let result = (self.value(), self.next);
        memory_allocator.free(mem::size_of::<FreeList<T>>());
        result
    }
}

impl<T> fmt::Display for FreeList<T> where T : Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"")
    }
}

/*
pub struct FreeListIterator<T> {
    current: Option<Box<FreeList<T>>>,
}

impl<T> FreeListIterator<T> {
    pub fn new(head: Box<FreeList<T>>) -> FreeListIterator<T> {
        FreeListIterator { current: Some(head) }
    }
}

impl<T> iter::Iterator for FreeListIterator<T> {
    type Item = Box<T>;

    fn next(&mut self) -> Option<Box<T>> {
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