use core::marker;
use core::mem;
use core::ops;
use core::iter;
use core::ptr;
use MemoryAllocator;
use smart_ptr;
use stdx::Sequence;
use stdx::Iterable;

pub struct Array<T> {
    length : usize,
    start_address : usize,    
    phantom : marker::PhantomData<T>
}

impl <T> Array<T> {
     pub fn new<A>(length : usize, memory_allocator : &mut A) -> Array<T> where A : MemoryAllocator {

        let result = Array::<T>::new0(length, memory_allocator);
        Array::<T>::zero_memory(result.start_address(), result.mem_size());

        result
    }
        
    pub fn new_fill<A, F>(length : usize, filler : F, memory_allocator : &mut A) -> Self 
    where A : MemoryAllocator,
          F : Fn() -> T
    {
        let mut result = Array::<T>::new0(length, memory_allocator);
        result.fill(filler);

        result
    }

    fn zero_memory(start_address : usize, array_size : usize) {
        for i in 0..array_size {
            let address = start_address + i;
            unsafe { ptr::write_unaligned(address as *mut u8, 0); }
        }
    }

    fn new0<A>(length : usize, memory_allocator : &mut A) -> Self
    where A : MemoryAllocator {
        let size          = Array::<T>::mem_size_for(length);
        let start_address = memory_allocator.allocate(size).expect("No memory for Array");

        Array { 
            length : length,
            start_address : start_address,
            phantom : marker::PhantomData
        }
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

    pub fn elem_read(&self, index : usize) -> T {
        assert!(index < self.length);        
        unsafe { ptr::read(self.index_to_address(index) as *const T) }
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
        assert!(index < self.length as isize && index > -1);
        unsafe { &*(self.index_to_address_i(index) as *const T) }
    }

    pub fn elem_ref_mut_i(&self, index : isize) -> &mut T {
        assert!(index < self.length as isize && index > -1);
        unsafe { &mut *(self.index_to_address_i(index) as *mut T) }
    }

    pub fn indices(&self) -> IndicesIterator {
        IndicesIterator::new(self)
    }

    pub fn fill<F>(&mut self, mut filler : F) where F : FnMut() -> T {
        let addresses = self.indices().map(|i| self.index_to_address(i));

        for address in addresses {
            unsafe { ptr::write_unaligned(address as *mut T, filler()); }
        }
    }    

    /*
    pub fn fill_value1(&mut self, value : T) {
        let addresses = self.indices().map(|i| self.index_to_address(i));

        

        for address in addresses {
            unsafe { ptr::write_unaligned(address as *mut T, value); }
        }
    }
    */

    pub fn replace(&mut self, index : usize, value : T) -> T {
        unsafe {
            mem::replace(&mut *(self.index_to_address(index) as *mut T), value)
        }
    }

    fn index_to_address(&self, index : usize) -> usize {
        self.start_address + (mem::size_of::<T>() * index)
    }

    fn index_to_address_i(&self, index : isize) -> isize {
        self.start_address as isize + (mem::size_of::<T>() as isize * index)
    }

    fn start_address(&self) -> usize {
        self.start_address
    }        
}

impl<T> Iterable for Array<T> {
    
    type Item = T;

    type IntoIter = ArrayIterator<T>;

    fn iterator(&self) -> ArrayIterator<T> {
        ArrayIterator::new(smart_ptr::Shared::new(self))
    }
}

impl<T> Sequence for Array<T> {
    
    fn length(&self) -> usize {
        self.length
    }
}

impl <T> Array<T> where T : Copy {
    pub fn value(&self, index : usize) -> T {        
        unsafe { *(self.index_to_address(index) as *mut T) }
    }

    pub fn fill_value(&mut self, value : T) {
        let addresses = self.indices().map(|i| self.index_to_address(i));

        for address in addresses {
            unsafe { ptr::write_unaligned(address as *mut T, value); }
        }
    }

    pub fn new_fill_value<A>(length : usize, value : T, memory_allocator : &mut A) -> Self 
    where A : MemoryAllocator 
    {
        let mut result = Array::<T>::new0(length, memory_allocator);
        result.fill_value(value);

        result
    }
}

impl <T> Array<T> where T : Default {
    pub fn fill_default(&mut self) {
        let addresses = self.indices().map(|i| self.index_to_address(i));

        for address in addresses {
            unsafe { ptr::write_unaligned(address as *mut T, T::default()); }
        }
    }

    pub fn new_fill_default<A>(length : usize, memory_allocator : &mut A) -> Self 
    where A : MemoryAllocator 
    {
        let mut result = Array::<T>::new0(length, memory_allocator);
        result.fill_default();

        result
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

pub struct ArrayIterator<T> {
    i : usize,
    array : smart_ptr::Shared<Array<T>>,    
}

impl<T> ArrayIterator <T> {

    pub fn new(array : smart_ptr::Shared<Array<T>>) -> Self {
        ArrayIterator {
            i  : 0,
            array : array,      
        }
    }
}

impl<T> iter::Iterator for ArrayIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.i >= self.array.length() {
            None
        }
        else {
            let result = self.array.elem_read(self.i);
            self.i += 1;

            Some(result)
        }
    }
}


//impl <'a, T> IteratorExt for ArrayIterator <'a, T> {}