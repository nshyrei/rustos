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