pub mod tags_info;
pub mod tag;
pub mod multiboot_header_tag;
pub mod tag_iterator;

use multiboot_header::tag::Tag;
use multiboot_header::tag_iterator::TagIterator;

#[repr(C)]
pub struct MultibootHeader {
    length: u32,
    resrved: u32,
    first_tag: Tag,
}

impl MultibootHeader {
    pub unsafe fn load(address: usize) -> &'static MultibootHeader {
        &(*(address as *const MultibootHeader))
    }

    pub fn start_address(&self) -> usize {
        self as *const _ as usize
    }

    pub fn end_address(&self) -> usize {
        (self as *const _ as usize) + self.length as usize
    }

    pub fn tags(&self) -> TagIterator {
        TagIterator::new(&self.first_tag as *const _ as usize)
    }

    pub fn read_tag<T>(&self) -> &'static T
        where T: multiboot_header_tag::MultibootHeaderTag
    {
        let mut tags = self.tags();
        let tag = tags.find(|t| t.tag_type == T::numeric_type()).unwrap();
        let tag_address = tag as *const _ as usize;
        unsafe { &(*(tag_address as *const T)) }
        //T::from_unsafe(tag as *const _ as usize)
    }
}