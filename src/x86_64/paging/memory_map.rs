//! The memory map has been pre-filled by our bootloader
//! This module allows us to access this memory map

use core::ptr::read_volatile;

const REGION_LENGTH: usize = 0x500;
const REGION_MAP: usize = 0x504;

#[derive(Debug)]
#[repr(C)]
pub struct Region {
    pub base_addr: usize,
    length: usize,
    region_type: u32,
}

impl Region {
    /// Returns the last valid region address
    pub fn end(&self) -> usize {
        self.base_addr + self.length - 1
    }
}

/// Returns the memory map's region count
pub fn region_count() -> u32 {
    unsafe {
        read_volatile(REGION_LENGTH as *const u32)
    }
}

/// Returns a specific region
pub fn get_region(index: u32) -> Region {
    let offset = 24 * index as usize;
    unsafe {
        read_volatile((REGION_MAP + offset) as *const Region)
    }
}