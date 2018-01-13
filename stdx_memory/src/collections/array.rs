use core::marker;
use core::mem;
use core::ops;
use core::iter;
use core::ptr;
use MemoryAllocator;
use smart_ptr;


pub struct Array<T> {
    length : usize,
    start_address : usize,    
    phantom : marker::PhantomData<T>
}

impl <T> Array<T> {
     pub fn new<A>(length : usize, memory_allocator : &mut A) -> Array<T> where A : MemoryAllocator {

        let size          = Array::<T>::mem_size_for(length);
        let start_address = memory_allocator.allocate(size).expect("No memory for Array");

        // zero array memory
        for i in 0..size {
            let address = start_address + i;
            unsafe { ptr::write_unaligned(address as *mut u8, 0); }
        }

        Array { 
            length : length,
            start_address : start_address,
            phantom : marker::PhantomData
        }
    }
    
    pub fn mem_size_for(length : usize) -> usize {
        mem::size_of::<T>() * length
    }

    pub fn mem_size(&self) -> usize {
        mem::size_of::<T>() * self.length
    }

    pub fn length(&self) -> usize {
        self.length
    }
    
    pub fn update(&mut self, index : usize, value : T) {
        assert!(index < self.length);

        let start_address = self.start_address;
        let entry_address = start_address + (mem::size_of::<T>() * index); 
        
        unsafe { ptr::write_unaligned(entry_address as *mut T, value); }
    }    

    pub fn free<A>(self, memory_allocator : &mut A) where A : MemoryAllocator {
        memory_allocator.free(self.start_address)
    }

    pub fn iterator(&self) -> ArrayIterator<T> {
        ArrayIterator::new(self)
    }

    pub fn elem_ref(&self, index : usize) -> &T {
        assert!(index < self.length);
        unsafe { &*(self.index_to_address(index) as *const T) }
    }

    pub fn elem_ref_mut(&self, index : usize) -> &mut T {
        assert!(index < self.length);
        unsafe { &mut *(self.index_to_address(index) as *mut T) }
    }

    pub fn elem_ref_i(&self, index : isize) -> &T {
        assert!(index < self.length as isize && index > 0);
        unsafe { &*(self.index_to_address_i(index) as *const T) }
    }

    pub fn elem_ref_mut_i(&self, index : isize) -> &mut T {
        assert!(index < self.length as isize && index > 0);
        unsafe { &mut *(self.index_to_address_i(index) as *mut T) }
    }

    pub fn indices(&self) -> IndicesIterator {
        IndicesIterator::new(self)
    }    

    fn index_to_address(&self, index : usize) -> usize {
        self.start_address + (mem::size_of::<T>() * index)
    }

    fn index_to_address_i(&self, index : isize) -> isize {
        self.start_address as isize + (mem::size_of::<T>() as isize * index)
    }
}

impl <T> Array<T> where T : Copy {
    pub fn value(&self, index : usize) -> T {        
        unsafe { *(self.index_to_address(index) as *mut T) }
    }
}

impl <T> Array<T> where T : Default {
    pub unsafe fn fill_default(&self) {
        let addresses = self.indices().map(|i| self.index_to_address(i));

        for address in addresses {
            ptr::write_unaligned(address as *mut T, T::default());
        }
    }
}

impl<T> ops::Index<usize> for Array<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {        
        self.elem_ref(index)
    }
}

impl<T> ops::IndexMut<usize> for Array<T> {

    fn index_mut(&mut self, index: usize) -> &mut T {
        self.elem_ref_mut(index)
    }
}

impl<T> ops::Index<isize> for Array<T> {
    type Output = T;

    fn index(&self, index: isize) -> &T {        
        self.elem_ref_i(index)
    }
}

impl<T> ops::IndexMut<isize> for Array<T> {

    fn index_mut(&mut self, index: isize) -> &mut T {
        self.elem_ref_mut_i(index)
    }
}


pub struct IndicesIterator {
    i : usize,
    last_index : usize
}

impl IndicesIterator {
    pub fn new<T>(array : &Array<T>) -> Self {
        IndicesIterator {
            i          : 0,
            last_index : array.length() - 1
        }
    }
}

impl iter::Iterator for IndicesIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.i > self.last_index {
            None
        }
        else {
            let result = self.i;
            self.i += 1;

            Some(result)
        }
    }
}

pub struct ArrayIterator<'a, T> where T : 'a {
    i : usize,
    array : &'a Array<T>,    
}

impl<'a, T> ArrayIterator <'a, T> {

    pub fn new(array : &'a Array<T>) -> Self {
        ArrayIterator {
            i  : 0,
            array : array,      
        }
    }
}

impl<'a, T> iter::Iterator for ArrayIterator<'a, T> {
    type Item = smart_ptr::Unique<T>;

    fn next(&mut self) -> Option<smart_ptr::Unique<T>> {
        if self.i >= self.array.length() {
            None
        }
        else {
            let result = self.array.elem_ref(self.i);
            self.i += 1;

            Some(smart_ptr::Unique::new(result))
        }
    }
}


//impl <'a, T> IteratorExt for ArrayIterator <'a, T> {}