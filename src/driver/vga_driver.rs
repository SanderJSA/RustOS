use core::fmt;
use core::fmt::Write;
use utils::lazy_static::LazyStatic;

static WRITER : LazyStatic<Writer> = LazyStatic::new(Writer::new);

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
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
    White = 15
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct ScreenChar {
    char: u8,
    color_code: ColorCode
}

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    col_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer
}
impl Writer {
    pub fn new() -> Writer {
        Writer {
           col_pos: 0,
           color_code: ColorCode::new(Color::White, Color::Black),
           buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
       }
    }

    pub fn write_byte(&mut self, byte: u8){
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col_pos == BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.col_pos;

                let color_code = self.color_code;
                self.buffer.chars[row][col] = ScreenChar {char: byte, color_code};
                self.col_pos += 1;
            }

        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row - 1][col] = self.buffer.chars[row][col];
            }
        }
        let blank = ScreenChar { char: b' ', color_code: self.color_code};
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT - 1][col] = blank;
        }
        self.col_pos = 0;

    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.obtain().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::driver::vga_driver::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

use crate::test;

test! (write_byte {
    let mut writer = Writer::new();

    writer.write_byte('R' as u8);

    let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 1][0];
    assert!(screen_char.char == 'R' as u8);
});

test! (write_string {
    let mut writer = Writer::new();

    writer.write_str("\nRust is awesome\n").unwrap();

    let line = writer.buffer.chars[BUFFER_HEIGHT - 2];
    let expected = "Rust is awesome".as_bytes();
    for i in 1..expected.len() {
        assert!(line[i].char == expected[i]);
    }
});
