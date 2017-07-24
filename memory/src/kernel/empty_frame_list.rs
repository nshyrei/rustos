use core::iter;
use core::mem;
use core::ptr;
use core::fmt;
use frame::Frame;
use kernel::bump_allocator::BumpAllocator;

/*
    Linked list of free memory frames
*/
#[repr(C)]
pub struct EmptyFrameList {
    value: Frame,
    next: Option<&'static EmptyFrameList>,
}

impl EmptyFrameList {
    pub fn new_tail(value: Frame, KERNEL_BASIC_HEAP_ALLOCATOR : &mut BumpAllocator) -> &'static EmptyFrameList {
        unsafe {            
            let address = KERNEL_BASIC_HEAP_ALLOCATOR
                .allocate(mem::size_of::<EmptyFrameList>())
                .expect("Failed to allocate memory for EmptyFrameList node");
            let result = EmptyFrameList {
                value: value,
                next: None,
            };

            ptr::write(address as *mut EmptyFrameList, result);
            &(*(address as *const EmptyFrameList))
        }
    }

    pub fn new(value: Frame, next: Option<&'static EmptyFrameList>, KERNEL_BASIC_HEAP_ALLOCATOR : &mut BumpAllocator) -> &'static EmptyFrameList {
        unsafe {
            let address = KERNEL_BASIC_HEAP_ALLOCATOR
                .allocate(mem::size_of::<EmptyFrameList>())
                .expect("Failed to allocate memory for EmptyFrameList node");
            let result = EmptyFrameList {
                value: value,
                next: next,
            };

            ptr::write(address as *mut EmptyFrameList, result);
            &(*(address as *const EmptyFrameList))
        }
    }


    pub fn value(&self) -> Frame {
        self.value
    }

    pub fn next(&self) -> Option<&'static EmptyFrameList> {
        self.next
    }

    pub fn add(&'static self, value: Frame, KERNEL_BASIC_HEAP_ALLOCATOR : &mut BumpAllocator) -> &'static EmptyFrameList {
        unsafe {
            let address = KERNEL_BASIC_HEAP_ALLOCATOR
                .allocate(mem::size_of::<EmptyFrameList>())
                .expect("Failed to allocate memory for EmptyFrameList node");
            let result = EmptyFrameList {
                value: value,
                next: Some(self),
            };

            ptr::write(address as *mut EmptyFrameList, result);
            &(*(address as *const EmptyFrameList))
        }
    }

    pub fn take(&self, KERNEL_BASIC_HEAP_ALLOCATOR : &mut BumpAllocator) -> (Frame, Option<&'static EmptyFrameList>) {
        let result = (self.value, self.next);
        unsafe { KERNEL_BASIC_HEAP_ALLOCATOR.free(mem::size_of::<EmptyFrameList>()); };
        result
    }    
}

impl fmt::Display for EmptyFrameList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "")
    }
}

pub struct EmptyFrameListIterator {
    current: Option<&'static EmptyFrameList>,
}

impl EmptyFrameListIterator {
    pub fn new(head: &'static EmptyFrameList) -> EmptyFrameListIterator {
        EmptyFrameListIterator { current: Some(head) }
    }
}

impl iter::Iterator for EmptyFrameListIterator {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        match self.current {
            Some(li) => {
                self.current = li.next;
                Some(li.value)
            }
            None => None,
        }
    }
}