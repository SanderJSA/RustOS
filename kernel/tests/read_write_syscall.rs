#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

extern crate kernel;

use core::panic::PanicInfo;
use kernel::file_system::File;
use kernel::*;

#[no_mangle]
#[link_section = ".kernel_start"]
extern "C" fn _start() -> ! {
    arch::init();

    let mut file = File::create("test_file").unwrap();

    // Write
    let expected = "This is a string that will be saved to the drive";
    let (section1, section2) = expected.as_bytes().split_at(20);
    assert_eq!(file.write(section1), Some(section1.len()));
    assert_eq!(file.write(section2), Some(section2.len()));
    drop(file);

    let mut file = File::open("test_file").unwrap();

    // Read
    let mut actual = [0u8; 48];
    let (mut section1, mut section2) = actual.split_at_mut(30);

    assert_eq!(file.read(&mut section1), Some(section1.len()));
    assert_eq!(file.read(&mut section2), Some(section2.len()));

    assert_eq!(actual, expected.as_bytes());

    serial_println!("read_write_syscall: [OK]");
    exit_qemu(QemuExitCode::Success)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("read_write_syscall: [KO]");
    exit_qemu(QemuExitCode::Failure)
}
