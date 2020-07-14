//! ATA PIO driver to communicate with hard disk
//! Read, write and get stoage space on master drive
//! Limited to 28bit LBA

use x86_64::port;

const ATA_MASTER: u8 = 0xE0;
#[allow(dead_code)]
const ATA_SLAVE: u8 = 0xF0;

const STATUS_BUSY: u8 = 0x80;
const STATUS_READY: u8 = 0x40;

const READ_PIO: u8 = 0x20;
const WRITE_PIO: u8 = 0x30;
const CACHE_FLUSH: u8 = 0xE7;
const IDENTIFY: u8 = 0xEC;

const DATA_PORT: u16 = 0x1F0;
const COMMAND_PORT: u16 = 0x1F7;
const SECTOR_COUNT: u16 = 0x1F2;
const LBA_LOW: u16 = 0x1F3;
const LBA_MID: u16 = 0x1F4;
const LBA_HIGH: u16 = 0x1F5;
const DEVICE_SELECT: u16 = 0x1F6;

/// Read n sectors starting at lba and writes contents in dst
pub fn read_sectors(lba: usize, sectors: u8, mut dst: usize) {
    assert!(sectors > 0);
    set_up_drive(lba, sectors, ATA_MASTER);
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

/// writes n sectors starting at lba from src
pub fn write_sectors(lba: usize, sectors: u8, src: &[u8]) {
    assert!(sectors > 0);
    set_up_drive(lba, sectors, ATA_MASTER);
    port::outb(COMMAND_PORT, WRITE_PIO);

    for j in 0..sectors as usize {
        wait_ready();

        for i in (0..512).step_by(2) {
            port::outw(DATA_PORT, to_word(src, j * 512 + i))
        }
    }
    port::outb(DATA_PORT, CACHE_FLUSH);
}

/// Returns number of sectors accessible by 28LBA
pub fn get_storage() -> u32 {
    set_up_drive(0, 0, 0xA0);
    port::outb(COMMAND_PORT, IDENTIFY);

    // Check if drive exists
    if port::inb(COMMAND_PORT) == 0 {
        return 0;
    }

    while port::inb(COMMAND_PORT) & STATUS_BUSY != 0 { }

    // Check if it is an ATA Drive
    if port::inb(LBA_LOW) != 0 || port::inb(LBA_MID) != 0 {
        return 0;
    }

    let mut buffer = [0; 256];
    for i in 0..256 {
        buffer[i] = port::inw(COMMAND_PORT);
    }

    buffer[60] as u32 | (buffer[61] as u32) << 16
}

fn set_up_drive(lba: usize, sectors: u8, ata_drive: u8) {
    assert!(lba < 0x1000000); // lba must fit in 28bits
    wait_ready();

    port::outb(DEVICE_SELECT, (lba >> 24) as u8 | ata_drive);
    port::outb(SECTOR_COUNT, sectors);
    port::outb(LBA_LOW, lba as u8);
    port::outb(LBA_MID, (lba >> 8) as u8);
    port::outb(LBA_HIGH, (lba >> 16) as u8);
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

fn to_word(src: &[u8], index: usize) -> u16 {
    if index + 1 < src.len() {
        src[index] as u16 + ((src[index + 1] as u16) << 8)
    }
    else {
        src[index] as u16
    }
}

use crate::test;

test!(non_empty_ata {
    assert!(get_storage() != 0);
});