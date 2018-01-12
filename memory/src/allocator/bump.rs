use stdx_memory::MemoryAllocator;
use stdx_memory::ConstantSizeMemoryAllocator;
use core::marker;
use core::mem;

#[derive(Clone)]
pub struct ConstSizeBumpAllocator {
    current_pointer     : usize,
    start_address       : usize,
    pointer_end_address : usize,
    allocation_size     : usize
}

impl ConstSizeBumpAllocator {            

    pub fn current_pointer(&self) -> usize {
        self.current_pointer
    }

    pub fn total_blocks_count(&self) -> usize {
        self.end_address() / self.allocation_size
    }

    pub fn start_address(&self) -> usize {
        self.start_address
    }

    pub fn end_address(&self) -> usize {
        self.pointer_end_address - 1
    }

    pub fn from_address(address: usize, size : usize, allocation_size : usize) -> Self {
        ConstSizeBumpAllocator {
            current_pointer     : address, 
            start_address       : address, 
            pointer_end_address : address + size,
            allocation_size     : allocation_size
        }
    }

    pub fn from_address_for_type<T>(address: usize, size : usize) -> Self
    where Self : marker::Sized {
        let elem_size = mem::size_of::<T>();
        Self::from_address(address, size, elem_size)
    }

    pub fn from_address_for_type_multiple<T>(address: usize, elems_count : usize) -> Self
    where Self : marker::Sized {    
        let elem_size = mem::size_of::<T>();
        Self::from_address(address, elem_size * elems_count, elem_size)
    }
}

impl ConstantSizeMemoryAllocator for ConstSizeBumpAllocator {
        
    fn allocate_size(&mut self) -> Option<usize> {        
        if self.current_pointer + self.allocation_size > self.pointer_end_address {
            None
        }
        else {
            let result = self.current_pointer;
            self.current_pointer += self.allocation_size;

            Some(result)
        }        
    }

    fn free_size(&mut self, pointer : usize) {
        self.current_pointer -= self.allocation_size;
    }    
}

#[derive(Clone)]
pub struct BumpAllocator {
    current_pointer     : usize,
    start_address       : usize,
    pointer_end_address : usize,    
}

impl BumpAllocator {
    pub fn current_pointer(&self) -> usize {
        self.current_pointer
    }

    pub fn from_address(address: usize, size : usize) -> Self {
        BumpAllocator { 
            current_pointer     : address, 
            start_address       : address, 
            pointer_end_address : address + size,            
        }
    }

    pub fn start_address(&self) -> usize {
        self.start_address
    }

    pub fn end_address(&self) -> usize {
        self.pointer_end_address - 1
    }    
}

impl MemoryAllocator for BumpAllocator {
    
    fn allocate(&mut self, size: usize) -> Option<usize> {        
        if self.current_pointer + size > self.pointer_end_address {
            None
        }
        else {
            let result = self.current_pointer;
            self.current_pointer += size;

            Some(result)
        }
    }

    fn free(&mut self, size: usize) {
        self.current_pointer -= size;
    }    
}