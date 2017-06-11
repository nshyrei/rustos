pub enum TagType {
    End,
    BasicMemoryInfo,
    MemoryMap,
}

impl From<TagType> for u32 {
    fn from(tag_type: TagType) -> u32 {
        match tag_type {
            TagType::End => 0,
            TagType::BasicMemoryInfo => 4,
            TagType::MemoryMap => 6,
        }
    }
}
