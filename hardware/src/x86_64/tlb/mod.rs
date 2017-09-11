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