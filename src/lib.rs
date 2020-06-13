#![no_std]
#![cfg_attr(test, no_main)]
#![feature(llvm_asm)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate x86_64 as x86_64_crate;

pub mod driver;
mod tty;
mod utils;
mod x86_64;
mod memory;

#[cfg(test)]
use core::panic::PanicInfo;
use x86_64::*;
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
    loop {};
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("\nRunning {} tests", tests.len());
    for test in tests {
        test();
    }
}

/// Panic Handler for unit test runner
#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Test failed: {}", _info);
    loop {}
}

/// Macro to quickly create a unit test
#[macro_export]
macro_rules! test {
    ($name:tt $body:block) => {
    #[cfg(test)]
    #[test_case]
    fn $name() {
        crate::print!("Test {}: ", stringify!($name));
        $body
        crate::println!("[OK]");
        }
    }
}
