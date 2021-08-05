//! Implementation of a USTAR file system

use super::File;
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
        let ustar_indicator = [b'u', b's', b't', b'a', b'r', 0];
        Metadata {
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
            filename_prefix: [0; 155],
        }
    }

    pub fn is_file(&self) -> bool {
        let res = str::from_utf8(&self.ustar_indicator);
        res.is_ok() && res.unwrap() == "ustar\0"
    }
}

pub fn ls() {
    let mut addr = fs_start_lba();
    loop {
        let mut metadata = Metadata::empty();
        ata::read_sectors(addr, 1, any_as_u8_slice_mut(&mut metadata));

        if !metadata.is_file() {
            return;
        }

        // Remove nullbytes from name
        let mut i = 0;
        while i < 100 && metadata.name[i] != b'\0' {
            i += 1;
        }
        let name = &metadata.name[0..i];
        crate::println!("{}", str::from_utf8(name).unwrap());

        addr += ((metadata.size + 511) / BLOCK_SIZE) + 1;
    }
}

pub fn create_file(name: &str) -> File {
    // Find free spot
    let mut addr = fs_start_lba();
    loop {
        let mut tmp = Metadata::empty();
        ata::read_sectors(addr, 1, any_as_u8_slice_mut(&mut tmp));
        if !tmp.is_file() {
            break;
        }
        addr += ((tmp.size + BLOCK_SIZE - 1) / BLOCK_SIZE) + 1;
    }

    // Create metadata
    let mut metadata = Metadata::empty();
    metadata.name[..name.len()].copy_from_slice(name.as_bytes());

    // Write to disk
    ata::write_sectors(addr, 1, any_as_u8_slice(&metadata));

    File {
        size: 0,
        index: 0,
        data_addr: addr + BLOCK_SIZE,
    }
}

pub fn open(filename: &str) -> Option<File> {
    let mut addr = fs_start_lba();
    loop {
        let mut metadata = Metadata::empty();
        ata::read_sectors(addr, 1, any_as_u8_slice_mut(&mut metadata));

        if !metadata.is_file() {
            return None;
        }

        // Remove nullbytes from name
        let meta_name = str::from_utf8(&metadata.name).unwrap();
        let name = meta_name.split_once('\0').unwrap_or((meta_name, "")).0;
        if name == filename {
            return Some(File {
                size: metadata.size,
                index: 0,
                data_addr: addr + BLOCK_SIZE,
            });
        }

        addr += ((metadata.size + 511) / BLOCK_SIZE) + 1;
    }
}

/// A helper function that translate a given input to a &[u8]
fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
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
