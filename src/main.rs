#![no_std]
#![no_main]
use core::panic::PanicInfo;
mod driver;
mod utils;

// Define panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
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


    // Hang
    loop {}
}
