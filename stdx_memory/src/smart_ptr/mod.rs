use core::ptr::NonNull;
use core::marker::PhantomData;
use core::fmt;
use core::marker::Sized;
use core::ops;

// shamelless copy of rust core lib
// only change is a func that accepts & and not &mut
pub struct Unique<T: Sized> {
    pointer: NonNull<*mut T>,
    phantom: PhantomData<T>,
}

impl<T> Unique<T> where T : Sized {
    
    pub fn new(ptr: *const T) -> Self {
        unsafe {
            Unique { 
                pointer : NonNull::new(ptr as *mut _),
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

impl<T> fmt::Display for Unique<T> where T : Sized + fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.pointer().fmt(f)
    }
}

pub struct Shared<T: Sized> {
    pointer: NonNull<*mut T>,
    phantom: PhantomData<T>,
}

impl<T> Shared<T> where T : Sized {
    
    pub fn new(ptr: *const T) -> Self {        
        unsafe {
            Shared { 
                pointer : NonZero::new(ptr as *mut _),
                phantom : PhantomData
            }
        }
    }

    pub fn from_usize(ptr: usize) -> Self {        
        unsafe {
            Shared { 
                pointer : NonNull::new(ptr as *mut _),
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

impl<T> ops::Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.pointer.as_ref() }
    }
}

impl<T> ops::DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.pointer.as_mut() }
    }
}

impl<T> Clone for Shared<T> where T : Sized {
    fn clone(&self) -> Self {
        Shared::new(self.pointer())
    }
}

impl<T> Copy for Shared<T> where T : Sized  { }

impl<T> fmt::Display for Shared<T> where T : Sized + fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.pointer().fmt(f)
    }
}