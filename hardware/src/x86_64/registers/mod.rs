
/// Contains page-table root pointer.
pub fn cr3() -> u64 {
    let ret: u64;
    unsafe { asm!("mov %cr3, $0" : "=r" (ret)) };
    ret
}

/// Switch page-table PML4 pointer (level 4 page table).
///
/// # Safety
/// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
/// changing the page mapping.
pub unsafe fn cr3_write(val: u64) {
    asm!("mov $0, %cr3" :: "r" (val) : "memory");
}
