#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

extern crate kernel;

use core::panic::PanicInfo;
use core::ptr::read_volatile;
use kernel::*;

#[no_mangle]
#[link_section = ".kernel_start"]
extern "C" fn _start() -> ! {
    init();

    unsafe {
        read_volatile(0xFFFFFFFFFFF as *const u32);
    }

    serial_println!("Simple boot: [KO]");
    exit_qemu(QemuExitCode::Failure)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("Stack overflow: [OK]");
    exit_qemu(QemuExitCode::Success)
}
