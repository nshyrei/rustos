use stdx::conversion::FromUnsafe;
use core::ptr::read;

pub struct BasicMemoryInfo {
    pub memory_lower: u32,
    pub memory_upper: u32,
}

impl FromUnsafe<usize> for BasicMemoryInfo {
    unsafe fn from_unsafe(address: usize) -> BasicMemoryInfo {
        let (_, _, memory_lower, memory_upper) = read(address as *const (u32, u32, u32, u32));
        BasicMemoryInfo {
            memory_lower: memory_lower,
            memory_upper: memory_upper,
        }
    }
}