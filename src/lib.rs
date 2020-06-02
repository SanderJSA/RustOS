#![no_std]
#![cfg_attr(test, no_main)]
#![feature(llvm_asm)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate x86_64;

pub mod driver;
mod utils;

#[cfg(test)]
use core::panic::PanicInfo;
use driver::*;

pub fn init() {
    gdt::init();
    interrupt::init_idt();
    interrupt::init_pics();
    x86_64::instructions::interrupts::enable();
}

pub fn welcome_message() {
    println!("     .~~~~`\\~~\\
     ;       ~~ \\
     |           ;
 ,--------,______|---.
/          \\-----`    \\
`.__________`-_______-'
           {}\n", 1 as char);
    println!("Howdy, welcome to RustOS");
}

//
// Unit tests runner
//

#[cfg(test)]
#[no_mangle]
#[link_section = ".kernel_start"]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {};
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("\nRunning {} tests", tests.len());
    for test in tests {
        test();
        println!("ok");
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Test failed: {}", _info);
    loop {}
}

//
// Sanity checks
//

#[cfg(test)]
#[test_case]
fn trivial_success() {
    print!("Test trivial_success: ");
    assert!(1 == 1);
}

#[cfg(test)]
#[test_case]
fn trivial_fail() {
    print!("Test trivial_fail: ");
    assert!(false);
}