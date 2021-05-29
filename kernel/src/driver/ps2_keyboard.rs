use super::vga_driver::WRITER;
use core::sync::atomic::{AtomicBool, Ordering};
use core::{ptr, str};

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
        unsafe { cur >= ptr::read_volatile(&self.end) }
    }

    fn get_line(&mut self) -> &str {
        let mut cur = self.start;

        loop {
            // Spin while waiting for buffer to get more values
            while self.wait_for_more(cur) {
                cur = self.end;
            }

            if self.data[cur] == '\n' as u8 {
                let string = &self.data[self.start..cur + 1];
                self.start = cur + 1;
                return str::from_utf8(string).unwrap();
            }
            cur += 1;
        }
    }
}

static mut STDIN_BUFFER: Buffer = Buffer::new();

pub fn readline() -> &'static str {
    unsafe { STDIN_BUFFER.get_line() }
}

pub fn update_stdin(code: u8) {
    match code {
        0x0E => unsafe { STDIN_BUFFER.remove_char() },
        0x2A | 0x36 => SHIFTED.store(true, Ordering::SeqCst),
        0xAA | 0xB6 => SHIFTED.store(false, Ordering::SeqCst),
        _ => {
            if let Some(value) = parse_normal_char(code) {
                unsafe { STDIN_BUFFER.add_char(*value) };
            }
        }
    }
}

const UNSHIFTED_MAP: &[char] = &[
    '?', '?', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '?', '\t', 'q', 'w', 'e',
    'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', '?', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k',
    'l', ';', '\'', '`', '?', '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', '?', '*',
    '?', ' ',
];

const SHIFTED_MAP: &[char] = &[
    '?', '?', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '?', '\t', 'Q', 'W', 'E',
    'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n', '?', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K',
    'L', ':', '"', '~', '?', '|', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?', '?', '*', '?',
    ' ',
];

static SHIFTED: AtomicBool = AtomicBool::new(false);

fn parse_normal_char(code: u8) -> Option<&'static char> {
    if SHIFTED.load(Ordering::SeqCst) {
        SHIFTED_MAP.get(code as usize)
    } else {
        UNSHIFTED_MAP.get(code as usize)
    }
}
