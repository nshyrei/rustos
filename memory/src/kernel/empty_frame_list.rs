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
pub struct EmptyFrameList<'a> {
    value: Frame,
    next: Option<&'a EmptyFrameList<'a>>,
}

impl<'a> EmptyFrameList<'a> {
    pub fn new_tail(value: Frame, KERNEL_BASIC_HEAP_ALLOCATOR : &mut BumpAllocator) -> &'a EmptyFrameList<'a> {
        unsafe {            
            let address = KERNEL_BASIC_HEAP_ALLOCATOR
                .allocate(mem::size_of::<EmptyFrameList>())
                .expect("Failed to allocate memory for EmptyFrameList node");
            let result = EmptyFrameList {
                value: value,
                next: None,
            };

            ptr::write_unaligned(address as *mut EmptyFrameList, result);
            &(*(address as *const EmptyFrameList))
        }
    }

    pub fn new(value: Frame, next: Option<&'a EmptyFrameList>, KERNEL_BASIC_HEAP_ALLOCATOR : &mut BumpAllocator) -> &'a EmptyFrameList<'a> {
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

    pub fn next(&self) -> Option<&'a EmptyFrameList> {
        self.next
    }

    pub fn add(&'a self, value: Frame, KERNEL_BASIC_HEAP_ALLOCATOR : &mut BumpAllocator) -> &'a EmptyFrameList {
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

    pub fn take(&self, KERNEL_BASIC_HEAP_ALLOCATOR : &mut BumpAllocator) -> (Frame, Option<&'a EmptyFrameList>) {
        let result = (self.value, self.next);
        KERNEL_BASIC_HEAP_ALLOCATOR.free(mem::size_of::<EmptyFrameList>());
        result
    }    
}

impl<'a> fmt::Display for EmptyFrameList<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "")
    }
}

pub struct EmptyFrameListIterator<'a> {
    current: Option<&'a EmptyFrameList<'a>>,
}

impl<'a> EmptyFrameListIterator<'a> {
    pub fn new(head: &'a EmptyFrameList) -> EmptyFrameListIterator<'a> {
        EmptyFrameListIterator { current: Some(head) }
    }
}

impl<'a> iter::Iterator for EmptyFrameListIterator<'a> {
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