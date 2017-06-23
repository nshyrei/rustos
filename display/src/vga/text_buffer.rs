use vga::character::Character;

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(C)]
pub struct TextBuffer {
    chars: [[Character; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl TextBuffer {
    pub fn chars(&self) -> [[Character; BUFFER_WIDTH]; BUFFER_HEIGHT] {
        self.chars
    }
}
