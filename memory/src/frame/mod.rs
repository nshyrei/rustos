pub mod frame_allocator;

pub const FRAME_SIZE: usize = 4096;

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Copy)]
pub struct Frame {
    number: usize,
}

impl Frame {

    pub fn new(number : usize) -> Frame {
        Frame { number : number }
    }

    pub fn from_address(address: usize) -> Frame {
        Frame { number: address / FRAME_SIZE }
    }

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn address(&self) -> usize {
        self.number * FRAME_SIZE
    }

    // creates new frame with number = self.number + 1
    pub fn next(&self) -> Frame {
        Frame { number : self.number + 1}
    }
}