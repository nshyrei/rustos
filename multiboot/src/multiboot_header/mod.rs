pub mod tags_info;
pub mod tag;
pub mod multiboot_header_tag;
pub mod mutiboot_header;
pub mod tag_iterator;

use multiboot_header::tag::Tag;
use multiboot_header::tag_iterator::TagIterator;
use stdx::conversion::FromAddressToStaticRef;

//pub unsafe fn elf_sections1(multiboot_multiboot_address: usize)
//                          -> tags_info::elf_sections1::ElfSectionsTag {
//let info_address = read_tag(multiboot_multiboot_address, tags::TagType::ElfSections);

//let raw = tags_info::elf_sections1::ElfSectionsTag::from_unsafe1(info_address);
//let rarA = (raw as *const _ as u64);

//let r = tags_info::elf_sections1::ElfSectionsTag::from_unsafe(info_address);
//let ra = (&r as *const _ as u64);
//r
//}


#[repr(C)]
pub struct MultibootHeader {
    length: u32,
    resrved: u32,
    first_tag: Tag,
}

impl FromAddressToStaticRef for MultibootHeader {
    unsafe fn from_unsafe(address: usize) -> &'static MultibootHeader {
        &(*(address as *const MultibootHeader))
    }
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

    pub unsafe fn read_tag<T>(&self) -> &'static T
        where T: multiboot_header_tag::MultibootHeaderTag + FromAddressToStaticRef
    {
        let mut tags = self.tags();
        let tag = tags.find(|t| t.tag_type == T::numeric_type()).unwrap();

        T::from_unsafe(tag as *const _ as usize)
    }
}