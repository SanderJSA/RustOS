//! This module sets up interfaces to communicate with hardware

pub mod gdt;
pub mod interrupt;
pub mod pic;
pub mod port;
pub mod paging;
pub mod serial;

#[allow(dead_code)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

#[allow(dead_code)]
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    unsafe {
        // Qemu is set up to read the port 0xf4
        // exit_code is always valid
        port::outd(0xf4, exit_code as u32);
    }
    loop {}
}