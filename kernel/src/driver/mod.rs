//! This module provides functions to interface with devices

pub mod e1000;
pub mod ps2_keyboard;
pub mod vga_driver;

use crate::arch::pci::Device;

pub fn init(devices: &[Device]) {
    for device in devices.iter() {
        if device.class() == e1000::DEVICE_TYPE {
            e1000::init(device)
        }
    }
}
