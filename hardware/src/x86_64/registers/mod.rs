
/// Returns page table root pointer register value (CR3)
#[inline(always)]
pub fn cr3() -> u64 {
    let ret: u64;
    unsafe { asm!("mov %cr3, $0" : "=r" (ret)) };
    ret
}

/// Returns code segment register value (CS)
#[inline(always)]
pub fn cs() -> u16 {
    let mut result : u16 = 0;

    unsafe { asm!("mov %cs, $0" : "=r" (result)) }

    result
}

/// Switch page-table PML4 pointer (level 4 page table).
/// # Safety
/// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
/// changing the page mapping.
#[inline(always)]
pub unsafe fn cr3_write(val: u64) {
    asm!("mov $0, %cr3" :: "r" (val) : "memory");
}

#[inline(always)]
pub unsafe fn rflags_write(val : u64) { asm!("pushq $0; popfq" :: "r"(val) : "memory" "flags") }

/// Updates stack pointer register (ESP) with provided value
/// # Arguments
/// * `val` - new value of stack pointer
/// # Safety
/// You should definitely know what you are doing if you are changing stack pointer.
/// Incorrect stack value can lead to silent overwrites of other memory areas or unexpected page fault.
#[inline(always)]
pub unsafe fn sp_write(val : u32) { asm!("mov $0, %esp" :: "r" (val) : "memory") }

/// Returns stack pointer register value (ESP)
#[inline(always)]
pub fn sp_read() -> u32 {
    let mut result : u32 = 0;

    unsafe { asm!("mov %esp, $0" : "=r" (result)) }

    result
}