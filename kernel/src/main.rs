#![no_std]
#![no_main]

extern crate kernel;

use kernel::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    arch::init();
    run_tty();
    arch::halt()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    arch::halt()
}
