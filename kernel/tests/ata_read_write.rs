#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

extern crate kernel;

use core::panic::PanicInfo;
use kernel::*;

#[no_mangle]
#[link_section = ".kernel_start"]
extern "C" fn _start() -> ! {
    init();

    // Get ourselves a buffer to play around with
    let mut buffer = [0u8; 2048];

    // Read 4 sectors and check if first contains bootloader
    ata::read_sectors(0, 4, &mut buffer);
    assert_eq!(buffer[510], 0x55);
    assert_eq!(buffer[511], 0xAA);

    // Write ones in sector 0 and check
    for i in 0..512 {
        buffer[i] = 1;
    }
    write_and_check(&buffer, 0, 0);

    // Write i in sector 1 and check
    for i in 0..512 {
        buffer[i] = i as u8;
    }
    write_and_check(&buffer, 1, 1);

    serial_println!("ata_read_write: [OK]");
    exit_qemu(QemuExitCode::Success)
}

fn write_and_check(expected: &[u8], start_sector: usize, default_value: u8) {
    // Write buffer
    ata::write_sectors(start_sector, 1, &expected);

    // Read what was written
    let mut actual = [default_value; 512];
    ata::read_sectors(start_sector, 1, &mut actual);

    // Check if both buffer are the same
    for i in 0..512 {
        assert_eq!(expected[i], actual[i]);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("ata_read_write: [KO]");
    exit_qemu(QemuExitCode::Failure)
}
