use core::nonzero::NonZero;
use core::marker::PhantomData;
use core::fmt;
use core::marker::Sized;
use core::ops;

// shamelless copy of rust core lib
// only change is a func that accepts & and not &mut
#[repr(C)]
pub struct Unique<T: Sized> {
    pointer: NonZero<*mut T>,
    phantom: PhantomData<T>,
}

impl<T> Unique<T> where T : Sized {
    
    pub fn new(ptr: *const T) -> Self {
        unsafe {
            Unique { 
                pointer : NonZero::new(ptr as *mut _),
                phantom : PhantomData
            }
        }        
    }

    /// Dereferences the content.    
    pub fn pointer(&self) -> &T {
        unsafe { &*self.pointer.get() }
    }

    pub fn pointer_mut(&self) -> &mut T {
        unsafe { &mut *self.pointer.get() }
    }    
}

impl<T> ops::Deref for Unique<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.pointer.get() }
    }
}

impl<T> ops::DerefMut for Unique<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.pointer.get() }
    }
}

impl <T> Unique<T> where T : Copy {
    pub fn value(&self) -> T {
        unsafe { *self.pointer.get() } 
    }
}

impl<T> Clone for Unique<T> where T : Sized {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Unique<T> where T : Sized  { }

impl<T> fmt::Display for Unique<T> where T : Sized + fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.pointer().fmt(f)
    }
}