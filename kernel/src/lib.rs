#![no_std]
#![cfg_attr(test, no_main)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![warn(clippy::all)]

extern crate alloc;

pub mod arch;
pub mod driver;
#[allow(dead_code)]
pub mod file_system;
pub mod memory_manager;
mod tty;
mod utils;

pub use arch::{ata, serial};
pub use arch::{exit_qemu, QemuExitCode};
pub use tty::run_tty;

global_asm!(include_str!("bootloader/stage1.s"));
global_asm!(include_str!("bootloader/stage2.s"));

/// Unit test runner
#[cfg(test)]
mod test {
    use super::*;
    use core::any;
    use core::panic::PanicInfo;

    #[no_mangle]
    pub extern "C" fn _start() -> ! {
        arch::init();
        test_main();
        exit_qemu(QemuExitCode::Success)
    }

    /// Panic Handler for unit test runner
    #[panic_handler]
    pub fn panic(_info: &PanicInfo) -> ! {
        serial_println!("[KO]");
        exit_qemu(QemuExitCode::Failure)
    }

    /// Create a wrapper around tests so we can print the test name
    pub trait Testable {
        fn run(&self);
    }

    impl<T: Fn()> Testable for T {
        fn run(&self) {
            serial_print!("Test {}: ", any::type_name::<T>());
            self();
            serial_println!("[OK]");
        }
    }

    pub fn test_runner(tests: &[&dyn Testable]) {
        serial_println!("\nRunning {} tests", tests.len());
        for test in tests {
            test.run();
        }
    }
}
