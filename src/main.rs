#![no_std]
#![no_main]
use core::panic::PanicInfo;
use vga_driver::print_hello;

mod vga_driver;

// Define panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Entry point of our kernel
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print_hello();
    loop {}
}
