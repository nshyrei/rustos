use frame::Frame_Size;
use kernel::bump_allocator::BumpAllocator;
use core::mem;
use core::ptr;
use core::ops;


pub struct FrameBitMap {}

impl FrameBitMap {
    pub fn new(total_available_memory: usize,
               allocator: &mut BumpAllocator)
               -> &'static FrameBitMap {
        let frame_bitmap_entry_size = mem::size_of::<FrameBitMapEntry>();
        let frames_count = total_available_memory / Frame_Size;

        let bitmap_size_help = frames_count % frame_bitmap_entry_size;
        let bitmap_size = if bitmap_size_help > 0 {
            (frames_count / frame_bitmap_entry_size) + 1
        } else {
            frames_count / frame_bitmap_entry_size
        };

        let address = allocator.allocate(bitmap_size);
        for i in address..bitmap_size {
            unsafe { ptr::write(i as *mut u8, 0) }
        }

        unsafe { &(*(address as *const FrameBitMap)) }
    }
}


impl ops::Index<usize> for FrameBitMap {
    type Output = FrameBitMapEntry;

    fn index(&self, frame_number: usize) -> &'static FrameBitMapEntry {
        let start_address = &self as *const _ as usize;
        let index = frame_number / mem::size_of::<FrameBitMapEntry>();

        unsafe { &(*((start_address + index) as *const FrameBitMapEntry)) }
    }
}

#[repr(C)]
pub struct FrameBitMapEntry {
    value: u8,
}

impl FrameBitMapEntry {
    fn index_in_byte_field(frame_number: usize) -> usize {
        frame_number % mem::size_of::<FrameBitMapEntry>()
    }

    fn offset_count(frame_number: usize) -> usize {
        let index_in_byte_field = FrameBitMapEntry::index_in_byte_field(frame_number);
        mem::size_of::<FrameBitMapEntry>() - 1 - index_in_byte_field
    }

    pub fn is_in_use(&'static self, frame_number: usize) -> bool {
        let offset_count = FrameBitMapEntry::offset_count(frame_number);
        let bit_mask = (1 as u8) << offset_count;

        self.value & bit_mask > 0
    }

    pub fn set_in_use(&'static mut self, frame_number: usize) -> () {
        let offset_count = FrameBitMapEntry::offset_count(frame_number);
        let bit_mask = (1 as u8) << offset_count;

        self.value | bit_mask;
    }

    pub fn set_free(&'static mut self, frame_number: usize) -> () {
        let offset_count = FrameBitMapEntry::offset_count(frame_number);
        let bit_mask = !((1 as u8) << offset_count);

        self.value & bit_mask;
    }
}