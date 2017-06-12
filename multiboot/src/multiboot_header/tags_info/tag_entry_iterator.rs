use core::marker::PhantomData;
use core::ptr::read;

pub struct TagEntryIterator<T> {
    phantom: PhantomData<T>,
    entry_address: usize,
    tag_end_address: usize,
    entry_size: usize,
}

pub trait ITagEntryIterator<T> {
    fn entry_address(&self) -> usize;
    fn tag_end_address(&self) -> usize;
    fn entry_size(&self) -> usize;
    fn new_entry_address(&mut self, arg: usize) -> ();

    fn next(&mut self) -> Option<T> {
        if self.entry_address() >= self.tag_end_address() {
            None
        } else {
            let result = unsafe { Some(read(self.entry_address() as *const T)) };
            let new_entry_address = self.entry_address() + self.entry_size();
            self.new_entry_address(new_entry_address);
            result
        }
    }
}

impl<T> ITagEntryIterator<T> for TagEntryIterator<T> {
    fn entry_address(&self) -> usize {
        self.entry_address
    }
    fn tag_end_address(&self) -> usize {
        self.tag_end_address
    }
    fn entry_size(&self) -> usize {
        self.entry_size
    }
    fn new_entry_address(&mut self, arg: usize) -> () {
        self.entry_address = arg;
    }
}

impl<T> TagEntryIterator<T> {
    pub fn new(entry_address: usize,
               tag_end_address: usize,
               entry_size: usize)
               -> TagEntryIterator<T> {
        TagEntryIterator {
            phantom: PhantomData,
            entry_address: entry_address,
            tag_end_address: tag_end_address,
            entry_size: entry_size,
        }
    }
}