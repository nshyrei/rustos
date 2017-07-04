use core::iter;
use kernel::bump_allocator::BumpAllocator;
use core::mem;
use core::ptr;
/*
    Linked list of free memory frames
*/
#[repr(C)]
pub struct EmptyFrameList {
    value: u64,
    next: Option<&'static EmptyFrameList>,
}

impl EmptyFrameList {
    pub fn new(value: u64, kernel_heap_allocator: &mut BumpAllocator) -> &'static EmptyFrameList {
        unsafe {
            let address = kernel_heap_allocator.allocate(mem::size_of::<EmptyFrameList>());
            let result = EmptyFrameList {
                value: value,
                next: None,
            };

            ptr::write(address as *mut EmptyFrameList, result);
            &(*(address as *const EmptyFrameList))
        }
    }

    pub fn add(&'static self,
               value: u64,
               kernel_heap_allocator: &mut BumpAllocator)
               -> &'static EmptyFrameList {
        unsafe {
            let address = kernel_heap_allocator.allocate(mem::size_of::<EmptyFrameList>());
            let result = EmptyFrameList {
                value: value,
                next: Some(self),
            };

            ptr::write(address as *mut EmptyFrameList, result);
            &(*(address as *const EmptyFrameList))
        }
    }
}

pub struct EmptyFrameListIterator {
    current: Option<&'static EmptyFrameList>,
}

impl iter::Iterator for EmptyFrameListIterator {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        match self.current {
            Some(li) => {
                self.current = li.next;
                Some(li.value)
            }
            None => None,
        }
    }
}