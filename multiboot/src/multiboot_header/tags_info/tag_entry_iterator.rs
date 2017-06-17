use core::marker::PhantomData;
use core::ptr::read;
use core::iter;
use stdx::conversion::FromAddressToStaticRef;

pub struct TagEntryIterator<T>
    where T: FromAddressToStaticRef
{
    phantom: PhantomData<T>,
    entry_address: usize,
    tag_end_address: usize,
    entry_size: usize,
}

impl<T> TagEntryIterator<T>
    where T: FromAddressToStaticRef
{
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

impl<T> iter::Iterator for TagEntryIterator<T>
    where T: FromAddressToStaticRef + 'static
{
    type Item = &'static T;

    fn next(&mut self) -> Option<&'static T> {
        if self.entry_address >= self.tag_end_address {
            None
        } else {
            let result = unsafe { Some(T::from_unsafe(self.entry_address)) };
            self.entry_address += self.entry_size;
            result
        }
    }
}