#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
#[link_section = ".kernel_start"]
pub extern "C" fn _start() -> ! {
    rust_os::init();
    rust_os::welcome_message();

    loop {};
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rust_os::println!("{}", _info);
    loop {}
}
