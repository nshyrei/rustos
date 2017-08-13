use kernel::bump_allocator::BumpAllocator;
use core::ptr;
use core::fmt;
use core::iter;

const FRAME_BITMAP_ENTRY_SIZE: usize = 8; //number of bits in byte

pub struct FrameBitMap {    
    start_address : usize,
    size : usize
}

impl FrameBitMap {
    pub fn new(total_available_memory: usize,
               frame_size: usize,
               KERNEL_BASIC_HEAP_ALLOCATOR : &mut BumpAllocator)
               -> FrameBitMap {
        let frames_count = total_available_memory / frame_size;

        let bitmap_size_help = frames_count % FRAME_BITMAP_ENTRY_SIZE;
        let bitmap_size = if bitmap_size_help > 0 {
            (frames_count / FRAME_BITMAP_ENTRY_SIZE) + 1
        } else {
            frames_count / FRAME_BITMAP_ENTRY_SIZE
        };

        let address = unsafe { KERNEL_BASIC_HEAP_ALLOCATOR
            .allocate(bitmap_size)
            .expect("Failed to allocate memory for frame bitmap") };
            
        for i in address..(address + bitmap_size) {
            unsafe { ptr::write(i as *mut FrameBitMapEntry, FrameBitMapEntry::new()) }
        }

        FrameBitMap {
            start_address : address,
            size : bitmap_size
        }
        //unsafe { &(*(address as *const FrameBitMap)) }
    }

    //todo check for out of bounds
    fn index(&self, frame_number: usize) -> &'static mut FrameBitMapEntry {        
        let index = frame_number / FRAME_BITMAP_ENTRY_SIZE;

        unsafe { &mut (*((self.start_address + index) as *mut FrameBitMapEntry)) }
    }

    pub fn is_in_use(&self, frame_number: usize) -> bool {
        self.index(frame_number).is_in_use(frame_number)
    }

    pub fn set_in_use(&self, frame_number: usize) {
        self.index(frame_number).set_in_use(frame_number)
    }

    pub fn set_free(&self, frame_number: usize) {
        self.index(frame_number).set_free(frame_number)
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl fmt::Display for FrameBitMap {    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "")
    }    
}

#[repr(C)]
struct FrameBitMapEntry {
    value: u8,
}

impl FrameBitMapEntry {
    fn new() -> FrameBitMapEntry {
        FrameBitMapEntry { value: 0 }
    }

    fn index_in_byte_field(frame_number: usize) -> usize {
        frame_number % FRAME_BITMAP_ENTRY_SIZE
    }

    fn offset_count(frame_number: usize) -> usize {
        let index_in_byte_field = FrameBitMapEntry::index_in_byte_field(frame_number);
        FRAME_BITMAP_ENTRY_SIZE - 1 - index_in_byte_field
    }

    fn is_in_use(&self, frame_number: usize) -> bool {
        let offset_count = FrameBitMapEntry::offset_count(frame_number);
        let bit_mask = (1 as u8) << offset_count;

        self.value & bit_mask > 0
    }

    fn set_in_use(&mut self, frame_number: usize) {
        let offset_count = FrameBitMapEntry::offset_count(frame_number);
        let bit_mask = (1 as u8) << offset_count;

        self.value = self.value | bit_mask
    }

    fn set_free(&mut self, frame_number: usize) {
        let offset_count = FrameBitMapEntry::offset_count(frame_number);
        let bit_mask = !((1 as u8) << offset_count);

        self.value = self.value & bit_mask
    }
}

impl fmt::Display for FrameBitMapEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:b}", self.value)
    }
}