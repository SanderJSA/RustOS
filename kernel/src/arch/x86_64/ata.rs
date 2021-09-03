//! ATA PIO driver to communicate with hard disk
//! Read, write and get stoage space on master drive
//! Limited to 28bit LBA

use super::port;

const ATA_MASTER: u8 = 0xE0;
#[allow(dead_code)]
const ATA_SLAVE: u8 = 0xF0;

const DATA_PORT: u16 = 0x1F0;
const COMMAND_PORT: u16 = 0x1F7;
const SECTOR_COUNT: u16 = 0x1F2;
const LBA_LOW: u16 = 0x1F3;
const LBA_MID: u16 = 0x1F4;
const LBA_HIGH: u16 = 0x1F5;
const DEVICE_SELECT: u16 = 0x1F6;

const READ_PIO: u8 = 0x20;
const WRITE_PIO: u8 = 0x30;
const CACHE_FLUSH: u8 = 0xE7;
const IDENTIFY: u8 = 0xEC;

const STATUS_BUSY: u8 = 0x80;
const STATUS_READY: u8 = 0x40;

/// Read `sectors` sectors starting at `lba`
/// and writes contents in `dst`
pub fn read_sectors(lba: usize, sectors: u8, dst: &mut [u8]) {
    assert!(sectors > 0);
    set_up_drive(lba, sectors, ATA_MASTER);
    // SAFETY: Drive is ready and has lba and sectors given
    // COMMAND_PORT is a valid port
    // READ_PIO is a valid value
    unsafe {
        port::outb(COMMAND_PORT, READ_PIO);
    }

    for sector in 0..sectors as usize {
        wait_drive_ready();

        for byte in (0..512).step_by(2) {
            // SAFETY: DATA_PORT is a valid port
            // Drive has data data ready to be read
            let pair = unsafe { port::inw(DATA_PORT) };
            dst[sector * 512 + byte] = pair as u8;
            dst[sector * 512 + byte + 1] = (pair >> 8) as u8;
        }
    }
}

/// writes `sectors` sectors starting at `lba` from `src`
pub fn write_sectors(lba: usize, sectors: u8, src: &[u8]) {
    assert!(sectors > 0);
    set_up_drive(lba, sectors, ATA_MASTER);
    // SAFETY: Drive is ready and has lba and sectors given
    // COMMAND_PORT is a valid port
    // WRITE_PIO is a valid value
    unsafe {
        port::outb(COMMAND_PORT, WRITE_PIO);
    }

    for j in 0..sectors as usize {
        wait_drive_ready();

        for i in (0..512).step_by(2) {
            let value = to_word(src, j * 512 + i);
            // SAFETY: DATA_PORT is a valid port
            // Drive is writing data and has sectors left to write
            unsafe {
                port::outw(DATA_PORT, value);
            }
        }
    }

    // SAFETY: COMMAND_PORT is a valid port
    // CACHE_FLUSH is a valid value
    unsafe {
        port::outb(COMMAND_PORT, CACHE_FLUSH);
    }
}

/// Returns number of sectors accessible by 28LBA
pub fn get_storage() -> u32 {
    // send IDENTIFY command
    set_up_drive(0, 0, 0xA0);
    // SAFETY: Drive is ready and has lba and sectors set to 0
    // COMMAND_PORT is a valid port
    // IDENTIFY is a valid value
    unsafe {
        port::outb(COMMAND_PORT, IDENTIFY);
    }

    // Check if drive exists
    if read_drive_status() == 0 {
        return 0;
    }

    // Wait for drive to be done working
    while read_drive_status() & STATUS_BUSY != 0 {}

    // Check if it is an ATA Drive
    // SAFETY: LBA_LOW and LBA_MID are both valid ports
    unsafe {
        if port::inb(LBA_LOW) != 0 || port::inb(LBA_MID) != 0 {
            return 0;
        }
    }

    // Retrieve data generated
    let mut buffer = [0; 256];
    for word in &mut buffer {
        // SAFETY: there are 256 16bits values generated
        // Retrievable on COMMAND_PORT
        *word = unsafe { port::inw(COMMAND_PORT) };
    }

    // Get available storage for 28bit LBA
    buffer[60] as u32 | (buffer[61] as u32) << 16
}

fn set_up_drive(lba: usize, sectors: u8, ata_drive: u8) {
    assert!(lba < 0x1000000); // lba must fit in 28bits
    wait_drive_ready();

    // SAFETY: Drive is ready to receive commands
    // Ports are valid
    // LBA fits in 28bits => values are valid
    unsafe {
        port::outb(DEVICE_SELECT, (lba >> 24) as u8 | ata_drive);
        port::outb(SECTOR_COUNT, sectors);
        port::outb(LBA_LOW, lba as u8);
        port::outb(LBA_MID, (lba >> 8) as u8);
        port::outb(LBA_HIGH, (lba >> 16) as u8);
    }
}

fn wait_drive_ready() {
    while !is_drive_ready() {}
}

fn is_drive_ready() -> bool {
    // Poll 400ns
    for _ in 0..4 {
        read_drive_status();
    }

    // Check if drive is not busy and is ready
    let status = read_drive_status();
    status & STATUS_BUSY == 0 && status & STATUS_READY != 0
}

fn read_drive_status() -> u8 {
    // SAFETY: COMMAND_PORT is a valid port
    unsafe { port::inb(COMMAND_PORT) }
}

/// Translates two u8 in a slice into a u16
fn to_word(src: &[u8], index: usize) -> u16 {
    if index + 1 < src.len() {
        src[index] as u16 + ((src[index + 1] as u16) << 8)
    } else {
        src[index] as u16
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_case]
    fn non_empty_ata() {
        assert!(get_storage() != 0);
    }
}
