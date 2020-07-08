//! ATA PIO driver to communicate with hard disk
//!

use x86_64::port;

const STATUS_BUSY: u8 = 0x80;
const STATUS_READY: u8 = 0x40;

const READ_PIO: u8 = 0x20;
const WRITE_PIO: u8 = 0x30;

const COMMAND_PORT: u16 = 0x1F7;
const DATA_PORT: u16 = 0x1F0;

pub fn read_sectors(lba: usize, sectors: u8, mut dst: usize) {
    assert!(sectors > 0);
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
    port::outb(COMMAND_PORT, READ_PIO);
    for _ in 0..sectors {
        wait_ready();

        let buffer = dst as *mut u16;
        for i in 0..256 {
            unsafe {
                *buffer.offset(i) = port::inw(DATA_PORT);
            }
        }
        dst += 512;
    }
}

pub fn write_sectors(lba: usize, sectors: u8, src: &[u8]) {
    assert!(sectors > 0);
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
    port::outb(COMMAND_PORT, WRITE_PIO);
    for j in 0..sectors as usize {
        wait_ready();
        for i in 0..256 {
            port::outb(DATA_PORT, src[j * 512 + i])
        }
    }
}

fn wait_ready() {
    while !is_ready() {}
}

fn is_ready() -> bool {
    // Poll 400ns
    for _ in 0..4 {
        port::inb(COMMAND_PORT);
    }
    let status = port::inb(COMMAND_PORT);
    status & STATUS_BUSY == 0 && status & STATUS_READY != 0
}
