use core::iter;
use stdx::conversion::FromAddressToStaticRef;
use multiboot_header::tags_info::elf_sections::ElfSectionHeader;
use multiboot_header::tags_info::elf_sections::ElfSectionType;

pub struct ElfSectionsIterator {
    entry_address: usize,
    tag_end_address: usize,
    entry_size: usize,
}

impl ElfSectionsIterator {
    pub fn new(entry_address: usize,
               tag_end_address: usize,
               entry_size: usize)
               -> ElfSectionsIterator {
        ElfSectionsIterator {
            entry_address: entry_address,
            tag_end_address: tag_end_address,
            entry_size: entry_size,
        }
    }
}

impl iter::Iterator for ElfSectionsIterator {
    type Item = &'static ElfSectionHeader;

    fn next(&mut self) -> Option<&'static ElfSectionHeader> {
        if self.entry_address >= self.tag_end_address {
            None
        } else {
            let result = unsafe { ElfSectionHeader::from_unsafe(self.entry_address) };
            self.entry_address += self.entry_size;
            // skip unused
            // todo possibly replace with loop if this will compile to recursion
            if result.section_type == ElfSectionType::Unused as u32 {
                self.next()
            } else {
                Some(result)
            }
        }
    }
}