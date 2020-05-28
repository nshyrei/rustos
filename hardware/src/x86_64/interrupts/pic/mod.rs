use pic8259_simple::ChainedPics;

pub(crate) const PIC_1_OFFSET: u8 = 32;
pub(crate) const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub const unsafe fn new() -> ChainedPics {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
}

pub unsafe fn initialize(pic : &mut ChainedPics) {
    pic.initialize();
}