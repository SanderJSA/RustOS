use core::ptr;
use core::str;
use driver::vga_driver::WRITER;

const BUFFER_SIZE: usize = 2048;

struct Buffer {
    data: [u8; BUFFER_SIZE],
    start: usize,
    end: usize,
}

impl Buffer {
    const fn new() -> Buffer {
        Buffer {
            data: [' ' as u8; BUFFER_SIZE],
            start: 0,
            end: 0,
        }
    }

    fn add_char(&mut self, value: char) {
        if self.end == BUFFER_SIZE {
            panic!("Not enough space in buffer");
        }

        self.data[self.end] = value as u8;
        self.end += 1;
        WRITER.obtain().write_byte(value as u8)
    }

    fn remove_char(&mut self) {
        if self.end != self.start {
            self.end -= 1;
            WRITER.obtain().erase_byte()
        }
    }

    fn wait_for_more(&mut self, cur: usize) -> bool {
        unsafe {
            cur >= ptr::read_volatile(&self.end)
        }
    }

    fn get_line(&mut self) -> &str {
        let mut cur = self.start;

        loop {
            // Spin while waiting for buffer to get more values
            while self.wait_for_more(cur) {
                cur = self.end;
            }

            if self.data[cur] == '\n' as u8 {
                let string = &self.data[self.start .. cur + 1];
                self.start = cur + 1;
                return str::from_utf8(string).unwrap();
            }
            cur += 1;
        }
    }
}

static mut STDIN_BUFFER: Buffer = Buffer::new();

pub fn readline() -> &'static str{
    unsafe {
        STDIN_BUFFER.get_line()
    }
}

pub fn update_stdin(code: u8) {
    if code == 0x0E {
        unsafe { STDIN_BUFFER.remove_char() };
    }

    if let Some(value) = parse_normal_char(code) {
        unsafe { STDIN_BUFFER.add_char(value) };
    }
}

fn parse_normal_char(code: u8) -> Option<char> {
    match code {
        0x02 => Some('1'),
        0x03 => Some('2'),
        0x04 => Some('3'),
        0x05 => Some('4'),
        0x06 => Some('5'),
        0x07 => Some('6'),
        0x08 => Some('7'),
        0x09 => Some('8'),
        0x0A => Some('9'),
        0x0B => Some('0'),

        0x0C => Some('-'),
        0x0D => Some('='),
        0x0F => Some('\t'),

        0x10 => Some('q'),
        0x11 => Some('w'),
        0x12 => Some('e'),
        0x13 => Some('r'),
        0x14 => Some('t'),
        0x15 => Some('y'),
        0x16 => Some('u'),
        0x17 => Some('i'),
        0x18 => Some('o'),
        0x19 => Some('p'),
        0x1A => Some('['),
        0x1B => Some(']'),
        0x1C => Some('\n'),

        0x1E => Some('a'),
        0x1F => Some('s'),
        0x20 => Some('d'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x24 => Some('j'),
        0x25 => Some('k'),
        0x26 => Some('l'),
        0x27 => Some(';'),
        0x28 => Some('\''),
        0x29 => Some('`'),

        0x2B => Some('\\'),
        0x2C => Some('z'),
        0x2D => Some('x'),
        0x2E => Some('c'),
        0x2F => Some('v'),
        0x30 => Some('b'),
        0x31 => Some('n'),
        0x32 => Some('m'),
        0x33 => Some(','),
        0x34 => Some('.'),
        0x35 => Some('/'),
        0x36 => Some('*'),

        0x39 => Some(' '),

        _ => None
    }
}