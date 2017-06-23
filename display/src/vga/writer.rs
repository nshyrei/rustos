use vga::character::Character;
use vga::text_buffer::TextBuffer;
use vga::text_buffer::BUFFER_HEIGHT;
use vga::text_buffer::BUFFER_WIDTH;
use vga::color::Color;
use vga::color::ColorVariant;

const VGA_ADDRESS: usize = 0xb8000;

pub struct Writer {
    column_position: usize,
    buffer: &'static mut TextBuffer,
}

impl Writer {
    pub fn new() -> Writer {
        Writer {
            column_position: 0,
            buffer: unsafe { &mut (*(VGA_ADDRESS as *mut TextBuffer)) },
        }
    }

    pub fn print_char(&mut self, character: Character) -> () {
        if character.is_new_line() {
            self.new_line();
        } else {
            if self.column_position >= BUFFER_WIDTH {
                self.new_line();
            }

            let mut char_array = self.buffer.chars();
            char_array[BUFFER_HEIGHT - 1][self.column_position] = character;
            self.column_position += 1;
        }
    }

    pub fn println_char(&mut self, character: Character) -> () {
        self.new_line();
        self.print_char(character);
    }

    pub fn print_string(&mut self, string: &str) -> () {
        let string_as_chars =
            string
                .bytes()
                .map(|e| Character::new(e, Color::new(ColorVariant::Green, ColorVariant::Black)));
        for c in string_as_chars {
            self.print_char(c);
        }
    }

    pub fn println_string(&mut self, string: &str) -> () {
        self.new_line();
        self.print_string(string);
    }

    fn new_line(&mut self) -> () {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let mut char_array = self.buffer.chars();
                let buf = char_array[row][col];
                char_array[row - 1][col] = buf;
            }
        }

        self.column_position = 0;
        self.clear_row(BUFFER_HEIGHT - 1);
    }

    fn clear_row(&mut self, row: usize) -> () {
        for col in 0..BUFFER_WIDTH {
            let blank_char = Character::blank();
            let mut char_array = self.buffer.chars();
            char_array[row][col] = blank_char;
        }
    }
}