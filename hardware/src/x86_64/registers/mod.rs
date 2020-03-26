
/// Contains page-table root pointer.
pub fn cr3() -> u64 {
    let ret: u64;
    unsafe { asm!("mov %cr3, $0" : "=r" (ret)) };
    ret
}

/// Contains code segment register value.
pub fn cs() -> u16 {
    let mut result : u16 = 0;

    unsafe { asm!("mov %cs, $0" : "=r" (result)) }

    result
}

/// Switch page-table PML4 pointer (level 4 page table).
///
/// # Safety
/// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
/// changing the page mapping.
pub unsafe fn cr3_write(val: u64) {
    asm!("mov $0, %cr3" :: "r" (val) : "memory");
}

#[inline(always)]
pub unsafe fn rflags_write(val : u64) { asm!("pushq $0; popfq" :: "r"(val) : "memory" "flags") }

#[inline(always)]
pub unsafe fn sp_write(val : u32) { asm!("mov $0, %esp" :: "r" (val) : "memory") }

#[inline(always)]
pub unsafe fn pushq(val : u64) {
    asm!("pushq $0" :: "r" (val) : "memory");
}

#[inline(always)]
pub unsafe fn jump(val : u64) {
    asm!("pushq $0" :: "r" (val) : "memory");
    asm!("ret" :::);
}

#[inline(always)]
pub unsafe fn iret(val : u64) {
    //asm!("pushq $0" :: "r" (val) : "memory");
    asm!("iret" :::);
}