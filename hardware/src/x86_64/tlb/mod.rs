use x86_64::registers::cr3;
use x86_64::registers::cr3_write;

/// Cleares entry from Translation Lookaside Buffer (TLB)
///
/// # Arguments
/// * `virtual_address` - virtual address of the entry
pub fn flush(virtual_address : usize) {
    unsafe {
        /* Clobber memory to avoid optimizer re-ordering access before invlpg, which may cause nasty bugs. */
        asm!("INVLPG ($0)"
            : 
            : "r"(virtual_address)
            : 
            : "memory"
        );
    }
}

/// Cleares all entries from Translation Lookaside Buffer (TLB)
pub fn flush_all() {
    unsafe { cr3_write(cr3()) }
}