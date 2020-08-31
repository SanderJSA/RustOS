#![no_std]
#![cfg_attr(test, no_main)]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate rlibc;
extern crate x86_64 as x86_64_crate;

pub mod driver;
mod fs;
pub mod memory;
mod tty;
mod utils;
pub mod x86_64;

pub use tty::run_tty;
use x86_64::*;
pub use x86_64::{exit_qemu, QemuExitCode};

global_asm!(include_str!("bootloader/stage1.s"));
global_asm!(include_str!("bootloader/stage2.s"));

/// Initializes hardware
pub fn init() {
    gdt::init();
    unsafe {
        // IDT is valid
        interrupt::init_idt();
    }
    interrupt::init_pics();
    x86_64_crate::instructions::interrupts::enable();
}


/// Unit test runner
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
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

#[cfg(test)]
use core::panic::PanicInfo;

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
