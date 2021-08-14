//! Implementation of the file system syscalls

mod ustar;
use crate::arch::ata;
pub use ustar::ls;
use ustar::{Entry, ReadDir, BLOCK_SIZE};

pub fn read_dir(dir_name: &str) -> Option<ReadDir> {
    if dir_name == "/" {
        Some(ReadDir::root())
    } else {
        None
    }
}

pub struct File {
    index: usize,
    entry: Entry,
}

impl File {
    pub fn new(entry: Entry) -> File {
        File { index: 0, entry }
    }
    pub fn create(filename: &str) -> Option<File> {
        Some(File::new(ustar::create_file(filename)))
    }

    pub fn open(filename: &str) -> Option<File> {
        ustar::open(filename).map(File::new)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        self.read_generic(buf, ata::read_sectors)
    }

    fn read_generic<T>(&mut self, buf: &mut [u8], sector_reader: T) -> Option<usize>
    where
        T: Fn(usize, u8, &mut [u8]),
    {
        let len = buf.len().min(self.entry.size - self.index);
        let lba = self.entry.get_sector() + 1 + get_sector(self.index);
        let sectors = get_sector(self.index + len.saturating_sub(1)) - get_sector(self.index) + 1;

        let mut sector_buf = alloc::vec![0; BLOCK_SIZE * sectors];
        sector_reader(lba, sectors as u8, &mut sector_buf);
        let block_offset = self.index % BLOCK_SIZE;
        buf[..len].copy_from_slice(&sector_buf[block_offset..(len + block_offset)]);

        self.index += len;
        Some(len)
    }

    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let end = get_sector(self.index + buf.len().saturating_sub(1));
        let start = get_sector(self.index);
        let sectors = start - end + 1;
        let lba = self.entry.get_sector() + 1 + get_sector(self.index);
        let block_offset = self.index % BLOCK_SIZE;

        let mut sector_buf = alloc::vec![0; sectors * BLOCK_SIZE ];
        ata::read_sectors(lba, sectors as u8, &mut sector_buf);

        sector_buf[block_offset..block_offset + buf.len()].copy_from_slice(buf);
        ata::write_sectors(lba, sectors as u8, &sector_buf);

        self.index += buf.len();
        self.entry.size += buf.len();
        ata::write_sectors(
            self.entry.get_sector(),
            1,
            ustar::any_as_u8_slice(&self.entry),
        );
        Some(buf.len())
    }

    pub fn get_size(&self) -> usize {
        self.entry.size
    }
    pub fn get_path(&self) -> &str {
        &self.entry.get_name()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // Nothing needed for now
    }
}

fn get_sector(addr: usize) -> usize {
    addr / BLOCK_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    fn create_file(size: usize) -> File {
        let mut entry = Entry::default();
        entry.size = size;
        File::new(entry)
    }

    fn sector_reader(sectors: &[u8], mut lba: usize, nb_sectors: u8, buf: &mut [u8]) {
        //The first sector is normally reserved for the metadata, which we ignore here
        lba -= 1;
        let start_addr = lba * BLOCK_SIZE;

        for i in 0..(nb_sectors as usize * BLOCK_SIZE) {
            buf[i] = sectors[start_addr + i];
        }
    }

    #[test_case]
    fn read_one_block() {
        let mut file = create_file(512);
        let sectors: Vec<u8> = (0..512).map(|val| val as u8).collect();
        let sector_reader =
            |lba, nb_sectors, buf: &mut [u8]| sector_reader(&sectors, lba, nb_sectors, buf);

        let mut buf = [0; 512];
        assert_eq!(file.read_generic(&mut buf, sector_reader), Some(512));
        assert_eq!(buf, sectors.as_slice());
    }

    #[test_case]
    fn read_two_blocks() {
        let mut file = create_file(1024);
        let sectors: Vec<u8> = (0..1024).map(|val| val as u8).collect();
        let sector_reader =
            |lba, nb_sectors, buf: &mut [u8]| sector_reader(&sectors, lba, nb_sectors, buf);

        let mut buf = [0; 1024];
        assert_eq!(file.read_generic(&mut buf, sector_reader), Some(1024));
        assert_eq!(buf, sectors.as_slice());
    }

    #[test_case]
    fn two_reads() {
        let mut file = create_file(512);
        let sectors: Vec<u8> = (0..512).map(|val| val as u8).collect();
        let sector_reader =
            |lba, nb_sectors, buf: &mut [u8]| sector_reader(&sectors, lba, nb_sectors, buf);

        let mut buf = [0; 512];
        assert_eq!(file.read_generic(&mut buf[..100], sector_reader), Some(100));
        assert_eq!(
            file.read_generic(&mut buf[100..], sector_reader),
            Some(512 - 100)
        );
        assert_eq!(buf, sectors.as_slice());
    }

    #[test_case]
    fn two_reads_two_blocks() {
        let mut file = create_file(1024);
        let sectors: Vec<u8> = (0..1024).map(|val| val as u8).collect();
        let sector_reader =
            |lba, nb_sectors, buf: &mut [u8]| sector_reader(&sectors, lba, nb_sectors, buf);

        let mut buf = [0; 1024];
        assert_eq!(file.read_generic(&mut buf[..345], sector_reader), Some(345));
        assert_eq!(
            file.read_generic(&mut buf[345..], sector_reader),
            Some(1024 - 345)
        );
        assert_eq!(buf, sectors.as_slice());
    }

    #[test_case]
    fn short_read_across_section() {
        let mut file = create_file(1024);
        let sectors: Vec<u8> = (0..1024).map(|val| val as u8).collect();
        let sector_reader =
            |lba, nb_sectors, buf: &mut [u8]| sector_reader(&sectors, lba, nb_sectors, buf);

        let mut buf = [0; 1024];
        assert_eq!(file.read_generic(&mut buf[..500], sector_reader), Some(500));
        assert_eq!(
            file.read_generic(&mut buf[500..600], sector_reader),
            Some(600 - 500)
        );
        assert_eq!(
            file.read_generic(&mut buf[600..], sector_reader),
            Some(1024 - 600)
        );
        assert_eq!(buf, sectors.as_slice());
    }

    #[test_case]
    fn read_too_much() {
        let mut file = create_file(512);
        let sectors: Vec<u8> = (0..512).map(|val| val as u8).collect();
        let sector_reader =
            |lba, nb_sectors, buf: &mut [u8]| sector_reader(&sectors, lba, nb_sectors, buf);

        let mut buf = [0; 600];
        assert_eq!(file.read_generic(&mut buf, sector_reader), Some(512));
        assert_eq!(buf[..512], sectors);
    }

    #[test_case]
    fn read_zero() {
        let mut file = create_file(512);
        let sectors: Vec<u8> = (0..512).map(|val| val as u8).collect();
        let sector_reader =
            |lba, nb_sectors, buf: &mut [u8]| sector_reader(&sectors, lba, nb_sectors, buf);

        let mut buf = [0; 0];
        assert_eq!(file.read_generic(&mut buf, sector_reader), Some(0));
    }

    #[test_case]
    fn read_tricky() {
        let mut file = create_file(2000);
        let sectors: Vec<u8> = (0..2048).map(|val| val as u8).collect();
        let sector_reader =
            |lba, nb_sectors, buf: &mut [u8]| sector_reader(&sectors, lba, nb_sectors, buf);

        let mut buf = [0; 2000];
        assert_eq!(
            file.read_generic(&mut buf[0..300], sector_reader),
            Some(300 - 0)
        );
        assert_eq!(
            file.read_generic(&mut buf[300..613], sector_reader),
            Some(613 - 300)
        );
        assert_eq!(
            file.read_generic(&mut buf[613..1700], sector_reader),
            Some(1700 - 613)
        );
        assert_eq!(
            file.read_generic(&mut buf[1700..2000], sector_reader),
            Some(2000 - 1700)
        );
        assert_eq!(buf, sectors[..2000]);
    }
}
