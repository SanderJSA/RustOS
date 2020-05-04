#![no_std]
#![no_main]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate x86_64;

mod driver;
mod utils;

use core::panic::PanicInfo;
use driver::*;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}


#[no_mangle]
#[link_section = ".kernel_start"]
pub extern "C" fn _start() -> ! {
    init();
    welcome_message();

    #[cfg(test)]
    test_main();

    loop {}
}

fn init() {
    interrupt::init_idt();
    gdt::init();
}

fn welcome_message() {
    println!("     .~~~~`\\~~\\
     ;       ~~ \\
     |           ;
 ,--------,______|---.
/          \\-----`    \\
`.__________`-_______-'
           {}\n", 1 as char);
    println!("Howdy, welcome to RustOS");
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


