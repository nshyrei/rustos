pub mod frame_allocator;

pub const Frame_Size: usize = 4096;

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct Frame {
    number: usize,
}

impl Frame {

    pub fn new(number : usize) -> Frame {
        Frame { number : number }
    }

    pub fn from_address(address: usize) -> Frame {
        Frame { number: address / Frame_Size }
    }

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn address(&self) -> usize {
        self.number * Frame_Size
    }
}