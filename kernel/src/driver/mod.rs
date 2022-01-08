//! This module provides functions to interface with devices

pub mod e1000;
pub mod ps2_keyboard;
pub mod vga_driver;

use crate::arch::pci::Device;
use e1000::E1000;

pub static mut ETHERNET_DEVICE: Option<E1000> = None;

pub fn init(devices: &[Device]) {
    for device in devices.iter() {
        crate::serial_println!("class: {:?}, vendor: {:?}", device.class(), device.id());
        if device.class() == e1000::DEVICE_TYPE {
            e1000::init(device);
        }
    }
}
