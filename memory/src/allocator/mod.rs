#[macro_use]
pub mod buddy;
pub mod bump;
pub mod free_list;

pub mod slab;

use frame::Frame;

fn align_addresses(start_address1 : usize, end_address1 : usize) -> (usize, usize) {
    (
        Frame::address_align_up(start_address1),
        Frame::address_align_down(end_address1)
    )
}

fn total_memory(start_address : usize, end_address : usize) -> usize {
    end_address - start_address + 1
}