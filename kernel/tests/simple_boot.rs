#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

extern crate kernel;

use core::panic::PanicInfo;
use kernel::*;

#[no_mangle]
#[link_section = ".kernel_start"]
extern "C" fn _start() -> ! {
    init();
    println!("simple print");

    serial_println!("Simple boot: [OK]");
    exit_qemu(QemuExitCode::Success)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("Simple boot: [KO]");
    exit_qemu(QemuExitCode::Failure)
}
