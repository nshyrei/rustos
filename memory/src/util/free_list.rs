use core::iter;
use core::mem;
use core::fmt;
use allocator::MemoryAllocator;
use util::bump_allocator::BumpAllocator;
use stdx::ptr;
use core;

/*
    A classic cons list
*/

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FreeList<T> {
    value: T,
    next: Option<ptr::Unique<FreeList<T>>>,
}

impl<T> FreeList<T> {

    pub fn size() -> usize {
        mem::size_of::<T>() + mem::size_of::<Option<ptr::Unique<FreeList<T>>>>()    
    }

    pub fn new(value: T, memory_allocator : &mut BumpAllocator) -> ptr::Unique<FreeList<T>> {
        unsafe {            
            let address = memory_allocator
                .allocate(mem::size_of::<FreeList<T>>())
                .expect("Failed to allocate memory for FreeList node");
            let result = FreeList {
                value: value,
                next: None,
            };

            core::ptr::write_unaligned(address as *mut FreeList<T>, result);
            ptr::Unique::new(&*(address as *const FreeList<T>))
        }
    }

    pub fn value_ref(&self) -> ptr::Unique<T> {
        ptr::Unique::new(&self.value)
    }

    pub fn next(&self) -> Option<ptr::Unique<FreeList<T>>> {
        self.next
    }

    pub fn add(&self, value: T, memory_allocator : &mut BumpAllocator) -> ptr::Unique<FreeList<T>> {
        unsafe {
            let address = memory_allocator
                .allocate(mem::size_of::<FreeList<T>>())
                .expect("Failed to allocate memory for FreeList node");
            let result = FreeList {
                value: value,
                next: Some(ptr::Unique::new(self)),
            };

            core::ptr::write(address as *mut FreeList<T>, result);
            ptr::Unique::new(&*(address as *const FreeList<T>))
        }
    }    
}

impl <T> FreeList<T> where T : Clone {
    pub fn value_copy(&self) -> T {
        self.value.clone()
    }

    pub fn take(self, memory_allocator : &mut BumpAllocator) -> (T, Option<ptr::Unique<FreeList<T>>>) {
        let result = (self.value_copy(), self.next);
        memory_allocator.free(mem::size_of::<FreeList<T>>());
        result
    }
}

impl<T> fmt::Display for FreeList<T> where T : Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"")
    }
}

pub struct FreeListIterator<T> {
    current: Option<ptr::Unique<FreeList<T>>>,
}

impl<T> FreeListIterator<T> {
    pub fn new(head: &FreeList<T>) -> FreeListIterator<T> {
        FreeListIterator { current: Some(ptr::Unique::new(head)) }
    }
}

impl<T> iter::Iterator for FreeListIterator<T> {
    type Item = ptr::Unique<T>;

    fn next(&mut self) -> Option<ptr::Unique<T>> {
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