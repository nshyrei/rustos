pub mod MultiBootHeaderTags;


pub unsafe fn ReadTag1(address: usize, tagType: u32) -> MultiBootHeaderTags::MultiBootHeaderTag {
    let mut tagsAddress = address + 8;
    let mut tag = *(tagsAddress as *const (u32, u32)); // (type, size)

    while (tag.0 != tagType || tag.0 != 0) {
        tagsAddress = ((tag.1 + 7) & !7) as usize;
        tag = *(tagsAddress as *const (u32, u32));
    }

    ReadTag(tagsAddress, tagType)
}

unsafe fn ReadTag(address: usize, tagType: u32) -> MultiBootHeaderTags::MultiBootHeaderTag {
    match tagType {
        4 => {
            let tup = *(address as *const (u32, u32, u32, u32));
            MultiBootHeaderTags::MultiBootHeaderTag::BasicMemoryInfo {
                memoryLower: tup.2,
                memoryUpper: tup.3,
            }
        }
        6 => {
            let tup = *(address as *const (u32, u32, u32, u32));
            let mut entryAddress = address + 16; //4 u32 fields offset
            let mut entry = *(entryAddress as *const MultiBootHeaderTags::MemoryMapEntry);
            let mut memoryEntries: [MultiBootHeaderTags::MemoryMapEntry; 10] =
                [MultiBootHeaderTags::MemoryMapEntry::default(); 10];
            memoryEntries[0] = entry;

            let mut entryCounter = 1;
            while (entryAddress < address + tup.1 as usize) {
                entry = *((entryAddress + tup.2 as usize) as
                          *const MultiBootHeaderTags::MemoryMapEntry);
                memoryEntries[entryCounter] = entry;
                entryCounter += 1;
            }

            MultiBootHeaderTags::MultiBootHeaderTag::MemoryMap {
                entrySize: tup.1,
                entryVersion: tup.3,
                entries: memoryEntries,
            }

        }
        0 => MultiBootHeaderTags::MultiBootHeaderTag::End,
        _ => MultiBootHeaderTags::MultiBootHeaderTag::Unknown, 
    }
}
