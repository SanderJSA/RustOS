//! Implementation of a USTAR file system

use driver::ata;
use core::{str, mem, slice};

const LBA_START: usize = (128000 + 512) / 512;

#[allow(dead_code)]
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
struct Metadata {
    name: [u8; 100],
    permissions: u64,
    owner_id: u64,
    group_id: u64,
    size: usize,
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
    filename_prefix: [u8; 155],
}

impl Metadata {
    fn empty() -> Metadata {
        let ustar_indicator = ['u' as u8, 's' as u8, 't' as u8, 'a' as u8, 'r' as u8, 0];
        Metadata {
            name: [0; 100],
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
            filename_prefix: [0; 155],
        }
    }

    pub fn is_file(&self) -> bool {
        let res = str::from_utf8(&self.ustar_indicator);
        res.is_ok() && res.unwrap() == "ustar\0"
    }
}

pub fn ls() {
    let mut addr = LBA_START;
    loop {
        let mut metadata = Metadata::empty();
        ata::read_sectors(addr, 1, (&mut metadata as *mut _) as usize);

        if !metadata.is_file() {
            return;
        }

        // Remove nullbytes from name
        let mut i = 0;
        while i < 100 && metadata.name[i] != '\0' as u8 {
            i += 1;
        }
        let name = &metadata.name[0..i];
        crate::println!("{}", str::from_utf8(&name).unwrap());

        addr += ((metadata.size + 511) / 512) + 1;
    }
}

pub fn add_file(name: &str, data: &[u8], size: usize) {
    // Create metadata
    let mut metadata = Metadata::empty();
    metadata.size = size;
    for i in 0..name.len() {
        metadata.name[i] = name.as_bytes()[i];
    }

    // Find free spot
    let mut addr = LBA_START;
    loop {
        let mut tmp = Metadata::empty();
        ata::read_sectors(addr, 1, (&mut tmp as *mut _) as usize);
        if !tmp.is_file() {
            break
        }
        addr += ((tmp.size + 511) / 512) + 1;
    }

    // Write to disk
    ata::write_sectors(addr, 1, any_as_u8_slice(&metadata));

    if size > 0 {
        ata::write_sectors(addr + 512, ((size + 511) / 512) as u8, data);
    }
}

fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        // *T points to valid memory
        // size_of<T> guarantees our slice only contains T
        slice::from_raw_parts(
            (p as *const T) as *const u8,
            mem::size_of::<T>(),
        )
    }
}