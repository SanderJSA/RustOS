#![no_std]
#![no_main]

extern crate rust_os;

use core::panic::PanicInfo;
use rust_os::println;

#[no_mangle]
#[link_section = ".kernel_start"]
pub extern "C" fn _start() -> ! {
    rust_os::init();

    rust_os::run_tty();

    loop {};
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}
