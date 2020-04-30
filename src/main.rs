#![no_std]
#![no_main]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

mod driver;
mod utils;

use core::panic::PanicInfo;
use driver::*;


// Define panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}


// Entry point of our kernel
#[no_mangle]
#[link_section = ".kernel_start"]
pub extern "C" fn _start() -> ! {

    // Print Welcome message
    println!("     .~~~~`\\~~\\
     ;       ~~ \\
     |           ;
 ,--------,______|---.
/          \\-----`    \\
`.__________`-_______-'
           {}\n", 1 as char);
    println!("Howdy, welcome to RustOS");

    exit(ExitCode::Success);

    #[cfg(test)]
    test_main();

    // Hang
    loop {}
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


