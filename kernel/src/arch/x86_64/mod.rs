//! This module sets up interfaces to communicate with hardware

pub mod ata;
pub mod gdt;
pub mod interrupt;
pub mod paging;
pub mod pci;
pub mod pic;
pub mod port;
pub mod serial;

use core::arch::asm;

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
    halt()
}

pub fn halt() -> ! {
    loop {
        // SAFETY: Operation halts until next external interrupt
        unsafe {
            asm!("hlt");
        }
    }
}

pub fn magic_breakpoint() {
    // SAFETY: Operation triggers a magic breakpoint in bochs, does nothing otherwise
    unsafe {
        asm!("xchg bx, bx");
    }
}
