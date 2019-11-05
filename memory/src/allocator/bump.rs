use stdx_memory::MemoryAllocator;
use stdx_memory::ConstantSizeMemoryAllocator;
use stdx_memory::MemoryAllocatorMeta;
use core::marker;
use core::mem;
use core::ptr;
use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags::memory_map::*;
use multiboot::multiboot_header::tags::elf;

#[repr(C)]
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
        self.size() / self.allocation_size
    }

    pub fn start_address(&self) -> usize {
        self.start_address
    }

    pub fn end_address(&self) -> usize {
        self.pointer_end_address - 1
    }

    pub fn size(&self) -> usize {
        self.end_address() - self.start_address() + 1
    }

    pub fn from_size(address: usize, size : usize, allocation_size : usize) -> Self {
        ConstSizeBumpAllocator {
            current_pointer     : address, 
            start_address       : address, 
            pointer_end_address : address + size,
            allocation_size     : allocation_size
        }
    }

    pub fn from_address(address: usize, end_address : usize, allocation_size : usize) -> Self {
        ConstSizeBumpAllocator {
            current_pointer     : address,
            start_address       : address,
            pointer_end_address : end_address,
            allocation_size     : allocation_size
        }
    }

    pub fn from_address_for_type<T>(address: usize, size : usize) -> Self
    where Self : marker::Sized {
        let elem_size = mem::size_of::<T>();
        Self::from_size(address, size, elem_size)
    }

    pub fn from_address_for_type_multiple<T>(address: usize, elems_count : usize) -> Self
    where Self : marker::Sized {    
        let elem_size = mem::size_of::<T>();
        Self::from_size(address, elem_size * elems_count, elem_size)
    }

    pub fn increase_size(&mut self, size : usize) {
        self.pointer_end_address += size;
    }

    pub fn is_inside_address_space(&self, pointer : usize) -> bool {
        pointer >= self.start_address && pointer <= self.pointer_end_address
    }
}

impl MemoryAllocatorMeta for ConstSizeBumpAllocator {
    fn aux_data_structures_size(&self) -> usize {
        0
    }

    fn start_address(&self) -> usize {
        self.start_address()
    }

    fn end_address(&self) -> usize {
        self.end_address()
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

impl MemoryAllocatorMeta for BumpAllocator {

    fn start_address(&self) -> usize { self.start_address() }

    fn end_address(&self) -> usize {
        self.end_address()
    }

    fn aux_data_structures_size(&self) -> usize {
        0
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

/*
pub struct SafeBumpAllocator {
    kernel_end_address : usize,
    kernel_start_address : usize,
    multiboot_start_address: usize,
    multiboot_end_address : usize,
    current_pointer     : usize,
    start_address       : usize,
    pointer_end_address : usize,    
    current_memory_area : ptr::NonNull<MemoryMapEntry>,
    memory_map : ptr::NonNull<MemoryMap>
}

impl SafeBumpAllocator {

    pub fn new(multiboot_header: &MultibootHeader, address: usize, size : usize) -> Self {
        let elf_sections = multiboot_header.read_tag::<elf::ElfSections>()
            .expect("Cannot create bump allocator without multiboot elf sections");
        let memory_areas = multiboot_header.read_tag::<MemoryMap>()
            .expect("Cannot create bump allocator without multiboot memory map");        

        if elf_sections.entries().count() == 0 {
            panic!("No elf sections, cannot determine kernel code address");
        }

        if memory_areas.entries().count() == 0 {
            panic!("No available memory areas for address allocator");
        }
        
        let kernel_start_section = elf_sections.entries()
            .min_by_key(|e| e.start_address())
            .unwrap();
        let kernel_end_section = elf_sections.entries()
            .max_by_key(|e| e.end_address())
            .unwrap();

        let kernel_start_address = kernel_start_section.start_address() as usize;
        let kernel_end_address   = kernel_end_section.end_address() as usize;        
        let fitting_memory_area = SafeBumpAllocator::next_fitting_memory_area(&memory_areas, address)
            .expect("Provided address doesn't fit any existing memory areas");

        SafeBumpAllocator {            
            kernel_end_address  : kernel_end_address,
            kernel_start_address : kernel_start_address,
            multiboot_start_address: multiboot_header.start_address(),
            multiboot_end_address : multiboot_header.end_address(),
            current_pointer     : address, 
            start_address       : address, 
            pointer_end_address : address + size,
            current_memory_area : ptr::NonNull::from(fitting_memory_area),
            memory_map : ptr::NonNull::from(memory_areas)
        }        
    }

    fn step_over_reserved_memory_if_needed(&self, address1 : usize, size : usize) -> usize {
        // dont touch multiboot data
        let address = address1 + size;
        if address >= self.multiboot_start_address &&
           address <= self.multiboot_end_address {
            self.step_over_reserved_memory_if_needed(self.multiboot_end_address + 1, size) // in case next will touch kernel code
        }
        // dont touch kernel code
        else if address >= self.kernel_start_address &&
                address <= self.kernel_end_address {
            self.step_over_reserved_memory_if_needed(self.kernel_end_address + 1, size) // in case next will touch empty address list
        }
        else{
            address
        }
    }
    
    fn next_fitting_memory_area(memory_map : &MemoryMap, address : usize) -> Option<&'static MemoryMapEntry> {        
        memory_map
            .entries()
            .filter(|e| address <= e.end_address() as usize)
            .min_by_key(|e| e.base_address())            
    }
}

impl MemoryAllocator for SafeBumpAllocator {
    
    fn allocate(&mut self, size: usize) -> Option<usize> {
        unsafe {
        if self.current_pointer + size > self.pointer_end_address {
            None
        }
        else {
            let mut possible_result = self.step_over_reserved_memory_if_needed(self.current_pointer, size);
            let mut possible_end_address = possible_result + size;

            if possible_end_address > self.current_memory_area.as_ref().end_address() as usize {
                if let Some(memory_area) = SafeBumpAllocator::next_fitting_memory_area(self.memory_map.as_ref(), possible_end_address) {

                    self.current_memory_area = ptr::NonNull::from(memory_area);
                    possible_result          = self.current_memory_area.as_ref().base_address() as usize;
                    possible_end_address     = possible_result + size;
                }
                else {
                    return None
                }
            }

            if possible_end_address > self.pointer_end_address {
                None
            }
            else {
                self.current_pointer = possible_end_address;
                Some(possible_result)
            }     
        }
        }
    }    

    fn free(&mut self, size: usize) {
        self.current_pointer -= size;
    }

    fn start_address(&self) -> usize {
        unimplemented!()
    }

    fn end_address(&self) -> usize {
        unimplemented!()
    }

    fn aux_data_structures_size(&self) -> usize {
        unimplemented!()
    }
}*/
