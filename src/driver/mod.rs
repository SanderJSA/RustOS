pub mod vga_driver;
pub mod port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit(exit_code: ExitCode) {
    port::out(0xf4, exit_code as u32);
}
