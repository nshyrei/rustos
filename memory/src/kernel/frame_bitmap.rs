use frame::Frame_Size;
use kernel::bump_allocator::BumpAllocator;
use core::mem;
use core::ptr;
use core::ops;
use core::fmt;

const Frame_BitMap_Entry_Size: usize = 8; //number of bits in byte

pub struct FrameBitMap {}

impl FrameBitMap {
    pub fn new(total_available_memory: usize,
               frame_size: usize,
               allocator: &mut BumpAllocator)
               -> &'static FrameBitMap {
        let frames_count = total_available_memory / frame_size;

        let bitmap_size_help = frames_count % Frame_BitMap_Entry_Size;
        let bitmap_size = if bitmap_size_help > 0 {
            (frames_count / Frame_BitMap_Entry_Size) + 1
        } else {
            frames_count / Frame_BitMap_Entry_Size
        };

        let address = allocator.allocate(bitmap_size);
        for i in address..(address + bitmap_size) {
            unsafe { ptr::write(i as *mut FrameBitMapEntry, FrameBitMapEntry::new()) }
        }

        unsafe { &(*(address as *const FrameBitMap)) }
    }

    fn index(&self, frame_number: usize) -> &'static mut FrameBitMapEntry {
        let start_address = self as *const _ as usize;
        let index = frame_number / Frame_BitMap_Entry_Size;

        unsafe { &mut (*((start_address + index) as *mut FrameBitMapEntry)) }
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
        frame_number % Frame_BitMap_Entry_Size
    }

    fn offset_count(frame_number: usize) -> usize {
        let index_in_byte_field = FrameBitMapEntry::index_in_byte_field(frame_number);
        Frame_BitMap_Entry_Size - 1 - index_in_byte_field
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