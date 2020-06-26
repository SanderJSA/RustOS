#![no_std]
#![cfg_attr(test, no_main)]
#![feature(llvm_asm)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate x86_64 as x86_64_crate;

pub mod driver;
pub mod x86_64;
mod memory;
mod tty;
mod utils;

#[cfg(test)]
use core::panic::PanicInfo;
use x86_64::*;
pub use x86_64::{QemuExitCode, exit_qemu};
pub use tty::run_tty;

/// Initializes hardware
pub fn init() {
    gdt::init();
    interrupt::init_idt();
    interrupt::init_pics();
    x86_64_crate::instructions::interrupts::enable();
}


/// Unit test runner
#[cfg(test)]
#[no_mangle]
#[link_section = ".kernel_start"]
extern "C" fn _start() -> ! {
    init();
    test_main();
    exit_qemu(QemuExitCode::Success)
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("\nRunning {} tests", tests.len());
    for test in tests {
        test();
    }
}

/// Panic Handler for unit test runner
#[cfg(test)]
#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[KO]");
    exit_qemu(QemuExitCode::Failure)
}

/// Macro to quickly create a unit test
#[macro_export]
macro_rules! test {
    ($name:tt $body:block) => {
    #[cfg(test)]
    #[test_case]
    fn $name() {
        crate::serial_print!("Test {}: ", stringify!($name));
        $body
        crate::serial_println!("[OK]");
        }
    }
}
