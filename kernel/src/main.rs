#![no_std]
#![no_main]

extern crate kernel;

use core::panic::PanicInfo;
use kernel::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    arch::init();

    run_tty();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}
