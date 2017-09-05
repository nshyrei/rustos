use core::fmt;
use core::option;

#[derive(Copy, Clone)]
pub struct Option<T>(pub option::Option<T>);

impl<T> fmt::Display for Option<T> where T : fmt::Display {    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(ref e) => write!(f,"Option value : {} ", e),
            None => write!(f, "None")
        }        
    }
}

pub fn split_u64(value : u64) -> (u32, u32) {
    ((value >> 32) as u32, (value & 0x00000000FFFFFFFF) as u32)    
}