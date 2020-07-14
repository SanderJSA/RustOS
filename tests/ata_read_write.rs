#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

extern crate rust_os;

use core::panic::PanicInfo;
use rust_os::*;
use core::slice::from_raw_parts_mut;
use rust_os::x86_64::paging::tables::EntryFlag;

#[no_mangle]
#[link_section = ".kernel_start"]
extern "C" fn _start() -> ! {
    init();

    // Get ourselves a buffer to play around with
    let address = memory::mmap(None, EntryFlag::Writable as u64);
    let buffer = unsafe { from_raw_parts_mut(address as *mut u8, 4096) };

    // Read 4 sectors and check if first contains bootloader
    driver::ata::read_sectors(0, 4, address);
    assert_eq!(buffer[510], 0x55);
    assert_eq!(buffer[511], 0xAA);

    // Now write ones in sector 0
    for i in 0..512 {
        buffer[i] = 1;
    }
    driver::ata::write_sectors(0, 1, &buffer);

    // Now write zeros in buffer to prevent false positives
    for i in 0..512 {
        buffer[i] = 0;
    }

    // Read first sector and check if all ones
    driver::ata::read_sectors(0, 1, address);
    for i in 0..512 {
        assert_eq!(buffer[i], 1);
    }

    serial_println!("ata_read_write: [OK]");
    exit_qemu(QemuExitCode::Success)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("ata_read_write: [KO]");
    exit_qemu(QemuExitCode::Failure)
}