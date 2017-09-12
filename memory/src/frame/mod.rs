pub mod frame_allocator;

use core::fmt;

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

    // creates new frame with number = self.number + 1
    fn next(&self) -> Frame {
        Frame { number : self.number + 1}
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "number: {}",
               self.number)
    }
}