use super::port::{inb, outb};
use crate::utils::lazy_static::LazyStatic;
use core::fmt;
use core::fmt::Write;

const COM1: u16 = 0x3F8;

static WRITER: LazyStatic<Serial> = LazyStatic::new(|| {
    unsafe {
        // COM1 is a valid port
        Serial::new(COM1)
    }
});

struct Serial {
    port: u16,
}

impl Serial {
    #[allow(dead_code)]
    pub unsafe fn new(port: u16) -> Serial {
        outb(port + 1, 0x00); // Disable all interrupts
        outb(port + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(port + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        outb(port + 1, 0x00); //                  (hi byte)
        outb(port + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(port + 4, 0x0B); // IRQs enabled, RTS/DSR set
        Serial { port }
    }

    fn write_buf_empty(&self) -> bool {
        unsafe {
            // Serial has been initialized, so this is safe
            inb(self.port + 5) & 0x20 == 0
        }
    }

    pub fn write_byte(&self, char: u8) {
        while self.write_buf_empty() {}

        unsafe {
            // Serial has been initialized, so this is safe
            outb(self.port, char)
        }
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    WRITER.obtain().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
