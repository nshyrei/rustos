#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color {
    pub value: u8,
}

impl Color {
    pub fn new(foreground: ColorVariant, background: ColorVariant) -> Color {
        Color { value: (((background as u8) << 4) | foreground as u8) }
    }
}

#[allow(dead_code)]
#[repr(u8)]
pub enum ColorVariant {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}