mod tags_info;
mod tags;
use stdx::conversion::FromUnsafe;

pub unsafe fn basic_memory_info(multiboot_multiboot_address: usize) -> tags_info::BasicMemoryInfo {
    let info_address = read_tag(multiboot_multiboot_address, tags::TagType::BasicMemoryInfo);
    tags_info::BasicMemoryInfo::from_unsafe(info_address)
}

pub unsafe fn memory_map(multiboot_multiboot_address: usize) -> tags_info::MemoryMap {
    let info_address = read_tag(multiboot_multiboot_address, tags::TagType::MemoryMap);
    tags_info::MemoryMap::from_unsafe(info_address)
}

unsafe fn read_tag(multiboot_address: usize, tag_type: tags::TagType) -> usize {
    let tag_type_as_int = u32::from(tag_type);
    let tag_type_end = u32::from(tags::TagType::End);

    let mut tags_multiboot_address = multiboot_address + 8;
    let mut tag = *(tags_multiboot_address as *const (u32, u32)); // (type, size)

    while tag.0 != tag_type_as_int && tag.0 != tag_type_end {
        tags_multiboot_address = tags_multiboot_address + ((tag.1 + 7) & !7) as usize;
        tag = *(tags_multiboot_address as *const (u32, u32));
    }

    tags_multiboot_address
}