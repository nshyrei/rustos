use core::iter;
use multiboot_header::tag::Tag;

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