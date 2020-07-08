//! ATA PIO driver to communicate with drives

use x86_64::port;

use crate::println;

const STATUS_BUSY: u8 = 0x80;
const STATUS_READY: u8 = 0x40;

const READ_PIO: u8 = 0x20;

pub fn read_sectors(lba: usize, sectors: u8, mut dst: usize) {
    wait_ready();

    // Send bit 24 - 27 of LBA and set LBA mode
    let upper = (lba >> 24) | 0xE0;
    port::outb(0x1F6, upper as u8);

    // Send number of sectors to read
    port::outb(0x1F2, sectors);

    // Send bits 0 - 23 of LBA
    port::outb(0x1F3, lba as u8);
    port::outb(0x1F4, (lba >> 8) as u8);
    port::outb(0x1F5, (lba >> 16) as u8);

    // Read
    port::outb(0x1F7, READ_PIO);
    for _ in 0..sectors {
        wait_ready();
        println!("ready to read");

        let buffer = dst as *mut u16;
        for i in 0..256 {
            unsafe {
                *buffer.offset(i) = port::inw(0x1F0);
            }
        }
        dst += 512;
    }
}

fn wait_ready() {
    while !is_ready() {}
}

fn is_ready() -> bool {
    // Poll 400ns
    for _ in 0..4 {
        port::inb(0x1F7);
    }
    let status = port::inb(0x1F7);
    status & STATUS_BUSY == 0 && status & STATUS_READY != 0
}
