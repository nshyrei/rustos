use MemoryAllocator;
use core::ptr;
use core::ops;
use core::ops::Deref;
use core::mem;
use core::cmp;
use core::cell;
use core::convert;

pub struct Box<T, A> where A : MemoryAllocator{
    unique           : ptr::NonNull<T>,
    memory_allocator : ptr::NonNull<A>

}

impl <T,A> Box<T,A> where A : MemoryAllocator {
    
    pub fn new(value : T, memory_allocator : &mut A) -> Self {
        let pointer = memory_allocator.allocate_for::<T>().expect("No memory for box value");
        
        unsafe {
            ptr::write_unaligned(pointer as *mut T, value);
            Box {
                unique           : ptr::NonNull::new_unchecked(pointer as *mut T),
                memory_allocator : ptr::NonNull::from(memory_allocator)
            }
        }
    }

    pub fn from_pointer(pointer : &T, memory_allocator : &mut A) -> Self {
        Box {
            unique           : ptr::NonNull::from(pointer),
            memory_allocator : ptr::NonNull::from(memory_allocator)
        }
    }

    fn allocator(&self) -> &ptr::NonNull<A> {
        &self.memory_allocator
    }
}

impl<T, A> ops::Deref for Box<T, A>  where A : MemoryAllocator {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.unique.as_ref() }
    }
}

impl<T, A> ops::DerefMut for Box<T, A> where A : MemoryAllocator {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.unique.as_mut() }
    }
}

impl<T, A> cmp::Ord for Box<T,A> where T : cmp::Ord, A : MemoryAllocator {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.deref().cmp(other.deref())
    }
}

impl<T, A> cmp::PartialOrd for Box<T, A> where T : cmp::PartialOrd, A : MemoryAllocator {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl<T, A> cmp::Eq for Box<T, A> where T : cmp::Eq, A : MemoryAllocator {

}

impl<T, A> cmp::PartialEq for Box<T, A> where T : cmp::PartialEq, A : MemoryAllocator {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

pub struct WeakBox<T>{
    unique : ptr::NonNull<T>
}

impl <T> WeakBox<T> {

    pub fn new<A>(value : T, memory_allocator : &mut A) -> Self  where A : MemoryAllocator {
        let pointer = memory_allocator.allocate_for::<T>().expect("No memory for box value");

        unsafe {
            ptr::write_unaligned(pointer as *mut T, value);
            WeakBox {
                unique : ptr::NonNull::new_unchecked(pointer as *mut T)
            }
        }
    }

    pub fn from_pointer(pointer : &T) -> Self {
        WeakBox {
            unique : ptr::NonNull::from(pointer)
        }
    }

    pub fn promote<A>(self, allocator : &mut A) -> Box<T, A> where A : MemoryAllocator {
        Box::from_pointer(self.deref(), allocator)
    }

    pub fn leak(self) -> T
    {
        unsafe {  ptr::read_unaligned(self.unique.as_ptr())  }
    }

}

impl<T> ops::Deref for WeakBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.unique.as_ref() }
    }
}

impl<T> ops::DerefMut for WeakBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.unique.as_mut() }
    }
}

impl<T> cmp::Ord for WeakBox<T> where T : cmp::Ord {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.deref().cmp(other.deref())
    }
}

impl<T> cmp::PartialOrd for WeakBox<T> where T : cmp::PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl<T> cmp::Eq for WeakBox<T> where T : cmp::Eq {

}

impl<T> cmp::PartialEq for WeakBox<T> where T : cmp::PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

pub struct SharedBox<T>{
    unique : ptr::NonNull<T>
}

impl <T> SharedBox<T> {
        
    pub fn new<A>(value : T, memory_allocator : &mut A) -> Self  where A : MemoryAllocator {
        let pointer = memory_allocator.allocate_for::<T>().expect("No memory for box value");
                
        unsafe {
            ptr::write_unaligned(pointer as *mut T, value); 
            SharedBox {
                unique : ptr::NonNull::new_unchecked(pointer as *mut T) 
            }
        }
    }

    pub fn pointer_equal(&self, other : &SharedBox<T>) -> bool {
        self.pointer() as *const T == other.pointer() as *const T
    }

    pub fn from_pointer(pointer : &T) -> Self {
        SharedBox {
            unique : ptr::NonNull::from(pointer)
        }
    }

    pub fn from_usize(pointer : usize) -> Self {
        unsafe { 
        SharedBox {
            unique : ptr::NonNull::new_unchecked(pointer as *mut T) 
        }}
    }

    pub fn pointer(&self) -> &T {
        unsafe { self.unique.as_ref() }
    }

    pub fn pointer_mut(&mut self) -> &mut T {
        unsafe { self.unique.as_mut() }
    }

}

impl<T> ops::Deref for SharedBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.unique.as_ref() }
    }
}

impl<T> ops::DerefMut for SharedBox<T> {
    fn deref_mut(&mut self) -> &mut T {
       unsafe { self.unique.as_mut() }
    }
}

impl<T> Clone for SharedBox<T> where T : Sized {
    fn clone(&self) -> Self {
        SharedBox::from_pointer(self.pointer())
    }
}

impl<T> Copy for SharedBox<T> where T : Sized  { }


pub struct RC<T, A> where A : MemoryAllocator {
    rc_box           : ptr::NonNull<RCBox<T>>,
    memory_allocator : ptr::NonNull<A>
}

impl<T, A> RC<T, A> where A : MemoryAllocator {
    pub fn new(value : T, memory_allocator : &mut A) -> Self {
        let pointer = memory_allocator.allocate_for::<RCBox<T>>().expect("No memory for RC box value");
        let rc_box = RCBox::new(value);

        unsafe { 
            ptr::write_unaligned(pointer as *mut RCBox<T>, rc_box);
            let rc_box = ptr::NonNull::new_unchecked(pointer as *mut RCBox<T>);
            let memory_allocator = ptr::NonNull::from(memory_allocator);

            RC {
                rc_box           : rc_box,
                memory_allocator : memory_allocator
            }
        }        
    }

    fn rc_box(&self) -> &RCBox<T> {
        unsafe { self.rc_box.as_ref() }
    }
}

impl<T, A> Clone for RC<T, A> where A : MemoryAllocator {
    fn clone(&self) -> Self {                
        unsafe {
            let pointer = self.rc_box.as_ptr() as usize;
            self.rc_box.as_ref().inc_ref_count();

            let rc_box     = ptr::NonNull::new_unchecked(self.rc_box.as_ptr());
            let memory_allocator = ptr::NonNull::new_unchecked(self.memory_allocator.as_ptr());

            RC {
                rc_box           : rc_box,
                memory_allocator : memory_allocator
            }
        }                
    }
}

impl<T, A> ops::Drop for RC<T, A> where A : MemoryAllocator {
    fn drop(&mut self) {
        unsafe {
            let pointer = self.rc_box.as_ptr() as usize;
            if self.rc_box.as_ref().reference_count() == 1 {
                let pointer = self.rc_box.as_ptr() as usize;
                self.memory_allocator.as_mut().free(pointer);
            }
            else {
                self.rc_box.as_ref().dec_ref_count();
            }
        }
    }
}

impl<T, A> ops::Deref for RC<T, A> where A : MemoryAllocator {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.rc_box.as_ref().value() }
    }
}

impl<T, A> ops::DerefMut for RC<T, A> where A : MemoryAllocator {
    fn deref_mut(&mut self) -> &mut T {
       unsafe { self.rc_box.as_mut().value_mut() }
    }
}

impl<T, A> cmp::Ord for RC<T, A> where T : cmp::Ord, A : MemoryAllocator {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.rc_box().cmp(other.rc_box())
    }
}

impl<T, A> cmp::PartialOrd for RC<T, A> where T : cmp::PartialOrd, A : MemoryAllocator {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.rc_box().partial_cmp(other.rc_box())
    }
}

impl<T, A> cmp::Eq for RC<T, A> where T : cmp::Eq, A : MemoryAllocator {

}

impl<T, A> cmp::PartialEq for RC<T, A> where T : cmp::PartialEq, A : MemoryAllocator {
    fn eq(&self, other: &Self) -> bool {
        self.rc_box().eq(other.rc_box())
    }
}

#[repr(C)]
struct RCBox<T> {
    value           : T,
    reference_count : cell::Cell<usize>
}

impl<T> RCBox<T> {

    fn value(&self) -> &T {
        &self.value
    }

    fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    fn new(value : T) -> Self {
        RCBox {
            value           : value,
            reference_count : cell::Cell::from(1)
        }
    }

    fn reference_count(&self) -> usize {
        self.reference_count.get()
    }

    fn dec_ref_count(&self) {
        let old = self.reference_count.get();
        self.reference_count.set(old - 1);
    }

    fn inc_ref_count(&self) {
        let old = self.reference_count.get();
        self.reference_count.set(old + 1);
    }
}

impl<T> cmp::Ord for RCBox<T> where T : cmp::Ord {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.value().cmp(other.value())
    }
}

impl<T> cmp::PartialOrd for RCBox<T> where T : cmp::PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value().partial_cmp(other.value())
    }
}

impl<T> cmp::Eq for RCBox<T> where T : cmp::Eq {

}

impl<T> cmp::PartialEq for RCBox<T> where T : cmp::PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.value().eq(other.value())
    }
}
