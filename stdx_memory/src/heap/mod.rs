use smart_ptr;
use MemoryAllocator;
use core::ptr;
use core::ops;
use core::ops::Deref;

pub struct Box<T>{
    unique : smart_ptr::Unique<T>
}

impl <T> Box<T> {
    
    pub fn new<A>(value : T, memory_allocator : &mut A) -> Box<T>  where A : MemoryAllocator {
        let pointer = memory_allocator.allocate_for::<T>().expect("No memory for box value");

        unsafe { ptr::write_unaligned(pointer as *mut T, value); }

        Box {
            unique : smart_ptr::Unique::new(pointer as *const T)
        }
    }    

    pub fn free<A>(self, memory_allocator : &mut A) where A : MemoryAllocator {
        memory_allocator.free(self.deref() as *const _ as usize)
    }

    pub fn from_pointer(pointer : &T) -> Self {
        Box {
            unique : smart_ptr::Unique::new(pointer)
        }
    }
}

impl<T> Box<T> where T : Clone {

    pub fn unbox<A>(self, memory_allocator : &mut A) -> T where A : MemoryAllocator {
        let result = self.deref().clone();
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
        let pointer = memory_allocator.allocate_for::<T>().expect("No memory for box value");
        unsafe { ptr::read_unaligned(pointer as *const T); }
        unsafe { ptr::write_unaligned(pointer as *mut T, value); }
        
        SharedBox {
            unique : smart_ptr::Shared::new(pointer as *const T)
        }
    }

    pub fn pointer_equal(&self, other : &SharedBox<T>) -> bool {
        self.pointer() as *const T == other.pointer() as *const T
    }

    pub fn from_pointer(pointer : &T) -> Self {
        SharedBox {
            unique : smart_ptr::Shared::new(pointer)
        }
    }

    pub fn from_usize(pointer : usize) -> Self {
        SharedBox {
            unique : smart_ptr::Shared::from_usize(pointer)
        }
    }

    pub fn pointer(&self) -> &T {
        self.unique.pointer()
    }

    pub fn pointer_mut(&self) -> &mut T {
        self.unique.pointer_mut()
    }

    pub fn free<A>(self, memory_allocator : &mut A) where A : MemoryAllocator {
        memory_allocator.free(self.pointer() as *const _ as usize)
    }

    pub fn read(&self) -> T {
        use core::ptr;
        unsafe { ptr::read_unaligned(self.pointer() as *const T) }
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
