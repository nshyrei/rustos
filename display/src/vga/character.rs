use vga::color::Color;
use vga::color::ColorVariant;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Character {
    pub character_code: u8,
    pub color: Color,
}

impl Character {
    pub fn new(character_code: u8, color: Color) -> Character {
        Character {
            character_code: character_code,
            color: color,
        }
    }

    pub fn blank() -> Character {
        Character {
            character_code: b' ',
            color: Color::new(ColorVariant::Black, ColorVariant::Black),
        }
    }

    pub fn new_line() -> Character {
        Character {
            character_code: b'\n',
            color: Color::new(ColorVariant::Black, ColorVariant::Black),
        }
    }

    pub fn is_new_line(&self) -> bool {
        self.character_code == b'\n'
    }
}