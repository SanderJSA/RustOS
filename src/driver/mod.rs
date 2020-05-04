pub mod vga_driver;
pub mod port;
pub mod interrupt;
pub mod cpuio;
pub mod gdt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum ExitCode {
    Success = 0x10,
    Failure = 0x11,
}

#[allow(dead_code)]
pub fn exit(exit_code: ExitCode) {
    port::outb(0xf4, exit_code as u8);
}
