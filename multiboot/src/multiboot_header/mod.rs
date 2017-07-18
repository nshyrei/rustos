pub mod tags_info;
pub mod tag;
pub mod tag_iterator;

use multiboot_header::tag::Tag;
use multiboot_header::tag_iterator::TagIterator;

pub trait MultibootHeaderTag {
    fn numeric_type() -> u32;
}

#[repr(C)]
pub struct MultibootHeader {
    length: u32,
    resrved: u32,
    first_tag: Tag,
}

impl MultibootHeader {
    pub fn load(address: usize) -> &'static MultibootHeader {
        unsafe { &(*(address as *const MultibootHeader)) }
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

    pub fn read_tag<T>(&self) -> Option<&'static T>
        where T: MultibootHeaderTag
    {
        let mut tags = self.tags();
        let tag = tags.find(|t| t.tag_type == T::numeric_type());
        tag.map(|e| {
            let tag_address = e as *const _ as usize;
            unsafe { &(*(tag_address as *const T)) }
        })        
    }
}