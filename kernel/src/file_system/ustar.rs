//! Implementation of a USTAR file system

use crate::arch::ata;
use core::{mem, slice, str};

pub const BLOCK_SIZE: usize = 512;

extern "C" {
    static _kernel_size: usize;
}

fn fs_start_lba() -> usize {
    unsafe {
        // Symbol is initialized by the linker
        (&_kernel_size as *const _ as usize / BLOCK_SIZE) + 2
    }
}

#[repr(u8)]
pub enum TypeFlag {
    File = 0,
    HardLink = 1,
    SymbolicLink = 2,
    CharacterDevice = 3,
    BlockDevice = 4,
    Directory = 5,
    Pipe = 6,
}

#[repr(C)]
#[repr(align(512))]
pub struct Entry {
    name: [u8; 100],
    permissions: u64,
    owner_id: u64,
    group_id: u64,
    pub size: usize,
    last_modified: [u8; 12],
    checksum: u64,
    type_flag: TypeFlag,
    linked_file: [u8; 100],
    ustar_indicator: [u8; 6],
    ustar_version: [u8; 2],
    owner: [u8; 32],
    group: [u8; 32],
    device_major_number: u64,
    device_minor_number: u64,
    ///Variation, the filename_prefix is slightly reduced
    filename_prefix: [u8; 147],
    ///To store entry's LBA
    sector: usize,
}

struct Directory {
    lba: usize,
}
impl Directory {
    pub fn new(lba: usize) -> Directory {
        Directory { lba }
    }
}

impl Iterator for Directory {
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        Entry::from_sector(self.lba).map(|entry| {
            self.lba += ((entry.size + BLOCK_SIZE - 1) / BLOCK_SIZE) + 1;
            entry
        })
    }
}

impl Entry {
    pub fn new(name: &str, sector: usize) -> Entry {
        let mut entry = Entry::default();
        entry.name[..name.len()].copy_from_slice(name.as_bytes());
        entry.sector = sector;
        entry
    }

    pub fn from_sector(lba: usize) -> Option<Entry> {
        let mut entry = Entry::default();
        ata::read_sectors(lba, 1, any_as_u8_slice_mut(&mut entry));
        match entry.is_file() {
            true => Some(entry),
            false => None,
        }
    }

    pub fn is_file(&self) -> bool {
        let res = str::from_utf8(&self.ustar_indicator);
        res.is_ok() && res.unwrap() == "ustar\0"
    }

    pub fn get_sector(&self) -> usize {
        self.sector
    }
}

impl Default for Entry {
    fn default() -> Entry {
        let ustar_indicator = [b'u', b's', b't', b'a', b'r', 0];
        Entry {
            name: [b'\0'; 100],
            permissions: 0,
            owner_id: 0,
            group_id: 0,
            size: 0,
            last_modified: [0; 12],
            checksum: 0,
            type_flag: TypeFlag::File,
            linked_file: [0; 100],
            ustar_indicator,
            ustar_version: [0; 2],
            owner: [0; 32],
            group: [0; 32],
            device_major_number: 0,
            device_minor_number: 0,
            filename_prefix: [0; 147],
            sector: 0,
        }
    }
}

fn str_from_cstring(cstr: &[u8]) -> &str {
    let len = cstr.iter().position(|c| c == &b'\0').unwrap_or(cstr.len());
    str::from_utf8(&cstr[..len]).expect("Could not parse Cstring")
}

pub fn ls() {
    let dir = Directory::new(fs_start_lba());
    for entry in dir {
        crate::println!("{}", str_from_cstring(&entry.name));
    }
}

pub fn create_file(name: &str) -> Entry {
    let lba = Directory::new(fs_start_lba())
        .last()
        .map(|entry| entry.sector + 1 + (entry.size + BLOCK_SIZE - 1) / BLOCK_SIZE)
        .unwrap_or_else(fs_start_lba);

    let entry = Entry::new(name, lba);
    ata::write_sectors(lba, 1, any_as_u8_slice(&entry));
    entry
}

pub fn open(filename: &str) -> Option<Entry> {
    Directory::new(fs_start_lba()).find(|entry| str_from_cstring(&entry.name) == filename)
}

/// A helper function that translate a given input to a &[u8]
pub fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        // *T points to valid memory
        // size_of<T> guarantees our slice only contains T
        slice::from_raw_parts((p as *const T) as *const u8, mem::size_of::<T>())
    }
}

/// A helper function that translate a given input to a &mut [u8]
fn any_as_u8_slice_mut<T: Sized>(p: &mut T) -> &mut [u8] {
    unsafe {
        // *T points to valid memory
        // size_of<T> guarantees our slice only contains T
        slice::from_raw_parts_mut((p as *mut T) as *mut u8, mem::size_of::<T>())
    }
}
