pub mod frame_allocator;

use core::fmt;
use core::iter;

pub const FRAME_SIZE: usize = 4096;

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Copy)]
pub struct Frame {
    number: usize,
}

impl Frame {

    pub fn from_address(address: usize) -> Frame {
        Frame { number: address / FRAME_SIZE }
    }

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn address(&self) -> usize {
        self.number * FRAME_SIZE
    }

    pub fn end_address(&self) -> usize {
        self.address() + FRAME_SIZE - 1
    }

    // creates inclusive range iterator
    pub fn range_inclusive(start_address : usize, end_address : usize) -> ExclusiveFrameRange {
        ExclusiveFrameRange::new(Frame::from_address(start_address), Frame::from_address(end_address))
    }

    fn is_frame_aligned(address : usize) -> bool {
        address % FRAME_SIZE == 0
    }

    fn address_to_frame_number(address : usize) -> usize {
        address / FRAME_SIZE
    }

    pub fn address_align_up(address : usize) -> usize {
        if Frame::is_frame_aligned(address) {
            address
        }
        else {
            (Frame::address_to_frame_number(address) + 1) * FRAME_SIZE
        }
    }

    pub fn address_align_down(address : usize) -> usize {
        if Frame::is_frame_aligned(address) {
            address
        }
        else {
            (Frame::address_to_frame_number(address) - 1) * FRAME_SIZE
        }
    }

    pub fn number_for_address(address : usize) -> usize {
        address / FRAME_SIZE
    }

    pub fn aligned_up(address : usize) -> Frame {
        if Frame::is_frame_aligned(address) {
            Frame::from_address(address)
        }
        else {
            Frame::from_address(address).next()
        }
    }

    pub fn aligned_down(address : usize) -> Frame {
        if Frame::is_frame_aligned(address) {
            Frame::from_address(address)
        }
        else {
            Frame::from_address(address).previous()
        }
    }

    // creates new frame with number = self.number + 1
    fn next(&self) -> Frame {
        Frame { number : self.number + 1}
    }

    fn previous(&self) -> Frame {
        let number = if self.number == 0 { 0 } else { self.number - 1 };
        Frame { number : number }
    }

    pub fn zero_frame(frame : &Frame) {
        use core::ptr;
        use frame::FRAME_SIZE;

        unsafe { ptr::write(frame.address() as *mut [u8; FRAME_SIZE], [0; FRAME_SIZE]); }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "number: {}",
               self.number)
    }
}

pub struct ExclusiveFrameRange {
    current_frame : Frame,
    end_frame : Frame    
}

impl ExclusiveFrameRange {
    fn new(current_frame : Frame, end_frame : Frame) -> ExclusiveFrameRange {
        ExclusiveFrameRange {
            current_frame : current_frame,
            end_frame : end_frame
        }
    }

    fn current_frame(&self) -> Frame {
        self.current_frame
    }

    fn end_frame(&self) -> Frame {
        self.end_frame
    }

    fn next_frame(&mut self) {
        self.current_frame = self.current_frame.next()
    }
}

impl iter::Iterator for ExclusiveFrameRange {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        let current_frame = self.current_frame();
        if current_frame <= self.end_frame() {            
            self.next_frame();
            Some(current_frame)
        }
        else {
            None
        }
    }
}