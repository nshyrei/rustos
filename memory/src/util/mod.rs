pub mod free_list;
pub mod bump_allocator;
pub mod frame_bitmap;
pub mod free_list_allocator;
pub mod buddy_allocator;
pub mod array;
pub mod double_linked_list;

use core::marker;
use stdx::smart_ptr;
use allocator::MemoryAllocator;
use core::ptr::write_unaligned;
use core::ops;

pub struct Box<T>{
    unique : smart_ptr::Unique<T>
}

impl <T> Box<T> {
    
    pub fn new<A>(value : T, memory_allocator : &mut A) -> Box<T>  where A : MemoryAllocator {
        let pointer = memory_allocator.allocate_from::<T>().expect("No memory for box value");

        unsafe { write_unaligned(pointer as *mut T, value); }

        Box {
            unique : smart_ptr::Unique::new(pointer as *const T)
        }
    }    

    pub fn free<A>(self, memory_allocator : &mut A) where A : MemoryAllocator {
        memory_allocator.free(self.pointer() as *const _ as usize)
    }

    fn from_pointer(pointer : &T) -> Self {
        Box {
            unique : smart_ptr::Unique::new(pointer)
        }
    }

    pub fn pointer(&self) -> &T {
        self.unique.pointer()
    }

    pub fn pointer_mut(&self) -> &mut T {
        self.unique.pointer_mut()
    }        
}

impl<T> Box<T> where T : Clone {

    pub fn unbox<A>(self, memory_allocator : &mut A) -> T where A : MemoryAllocator {
        let result = self.pointer().clone();
        self.free(memory_allocator);

        result
    }
}

impl<T> ops::Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.unique.pointer()
    }
}

impl<T> ops::DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.unique.pointer_mut()
    }
}

pub struct SharedBox<T>{
    unique : smart_ptr::Shared<T>
}

impl <T> SharedBox<T> {
    
    pub fn new<A>(value : T, memory_allocator : &mut A) -> Self  where A : MemoryAllocator {
        let pointer = memory_allocator.allocate_from::<T>().expect("No memory for box value");

        unsafe { write_unaligned(pointer as *mut T, value); }

        SharedBox {
            unique : smart_ptr::Shared::new(pointer as *const T)
        }
    }

    fn from_pointer(pointer : &T) -> Self {
        SharedBox {
            unique : smart_ptr::Shared::new(pointer)
        }
    }

    pub fn pointer(&self) -> &T {
        self.unique.pointer()
    }

    pub fn pointer_mut(&self) -> &mut T {
        self.unique.pointer_mut()
    }    
}

impl<T> ops::Deref for SharedBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.unique.pointer()
    }
}

impl<T> ops::DerefMut for SharedBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.unique.pointer_mut()
    }
}

impl<T> Clone for SharedBox<T> where T : Sized {
    fn clone(&self) -> Self {
        SharedBox::from_pointer(self.pointer())
    }
}

impl<T> Copy for SharedBox<T> where T : Sized  { }
