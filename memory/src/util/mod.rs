pub mod free_list;
pub mod bump_allocator;
pub mod frame_bitmap;
pub mod free_list_allocator;
pub mod buddy_allocator;
pub mod array;
pub mod double_linked_list;

use core::marker;
use stdx::ptr;
use allocator::MemoryAllocator;
use core::ptr::write_unaligned;
use core::ops;

pub struct Box<T>{
    unique : ptr::Unique<T>
}

impl <T> Box<T> {
    
    pub fn new<A>(value : T, memory_allocator : &mut A) -> Box<T>  where A : MemoryAllocator {
        let pointer = memory_allocator.allocate_from::<T>().expect("No memory for box value");

        unsafe { write_unaligned(pointer as *mut T, value); }

        Box {
            unique : ptr::Unique::new(pointer as *const T)
        }
    }

    pub fn from_pointer(pointer : &T) -> Box<T> {
        Box {
            unique : ptr::Unique::new(pointer)
        }
    }

    pub fn pointer(&self) -> &T {
        self.unique.pointer()
    }

    pub fn pointer_mut(&self) -> &mut T {
        self.unique.pointer_mut()
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