pub mod tags_info;
pub mod tags;
pub mod multiboot_header_tag;
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

pub unsafe fn read_tag<T>(multiboot_address: usize) -> &'static T
    where T: multiboot_header_tag::MultibootHeaderTag + FromAddressToStaticRef
{
    let tag_type_as_int = T::numeric_type();
    let tag_type_end = 0;
    let mut tags_multiboot_address = multiboot_address + 8;
    let mut tag = *(tags_multiboot_address as *const (u32, u32)); // (type, size)

    while tag.0 != tag_type_as_int && tag.0 != tag_type_end {
        tags_multiboot_address = tags_multiboot_address + ((tag.1 + 7) & !7) as usize;
        tag = *(tags_multiboot_address as *const (u32, u32));
    }

    T::from_unsafe(tags_multiboot_address)
}