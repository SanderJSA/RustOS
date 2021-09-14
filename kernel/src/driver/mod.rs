//! This module provides functions to interface with devices

pub mod intel_8254x;
pub mod ps2_keyboard;
pub mod vga_driver;

use crate::arch::pci::Device;
use alloc::vec::Vec;

pub fn init(devices: &Vec<Device>) {
    for device in devices.iter() {
        if device.class() == intel_8254x::DEVICE_TYPE {
            intel_8254x::init(device)
        }
    }
}
