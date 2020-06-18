//! This module sets up interfaces to communicate with hardware

pub mod gdt;
pub mod interrupt;
pub mod pic;
pub mod port;
pub mod paging;
pub mod serial;

#[allow(dead_code)]
#[repr(u8)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

#[allow(dead_code)]
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    port::outb(0xF4, exit_code as u8);
    loop {}
}