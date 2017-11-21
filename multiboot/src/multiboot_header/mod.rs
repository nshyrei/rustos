pub mod tags;
pub mod tag;
use core::iter;

pub trait MultibootHeaderTag {
    fn numeric_type() -> u32;
}

#[repr(C)]
pub struct MultibootHeader {
    length: u32,
    resrved: u32,
    first_tag: Tag,
}

#[repr(C)]
pub struct Tag {
    pub tag_type: u32,
    pub size: u32,
}

impl MultibootHeader {
    pub fn load(address: usize) -> &'static MultibootHeader {
        unsafe { &(*(address as *const MultibootHeader)) }
    }

    pub fn start_address(&self) -> usize {
        self as *const _ as usize
    }

    pub fn end_address(&self) -> usize {
        (self.start_address() + self.length as usize) - 1
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

pub struct TagIterator {
    entry_address: usize,
}

impl TagIterator {
    pub fn new(first_entry_address: usize) -> TagIterator {
        TagIterator { entry_address: first_entry_address }
    }
}

impl iter::Iterator for TagIterator {
    type Item = &'static Tag;

    fn next(&mut self) -> Option<&'static Tag> {
        let tag = unsafe { &(*(self.entry_address as *const Tag)) };
        if tag.tag_type == 0 {
            None
        } else {
            self.entry_address += ((tag.size + 7) & !7) as usize;
            Some(tag)
        }
    }
}