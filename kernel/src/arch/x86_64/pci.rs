use super::port;
use alloc::vec::Vec;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;
const ENABLE: u32 = 1 << 31;

const VENDOR_ID: u8 = 0;
const DEVICE_ID: u8 = 2;
const CLASS: u8 = 10;
const HEADER_TYPE: u8 = 14;

#[derive(Debug, Clone, Copy)]
enum DeviceType {
    EthernetController,
    VgaCompatibleController,
    HostBridge,
    IsaBridge,
    Unknown(u16),
}

impl DeviceType {
    pub fn new(class: u16) -> DeviceType {
        match class {
            0x02_00 => DeviceType::EthernetController,
            0x03_00 => DeviceType::VgaCompatibleController,
            0x06_00 => DeviceType::HostBridge,
            0x06_01 => DeviceType::IsaBridge,
            _ => DeviceType::Unknown(class),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Device {
    bus: u8,
    slot: u8,
    class: DeviceType,
}

impl Device {
    pub fn new(bus: u8, slot: u8, class: u16) -> Device {
        Device {
            bus,
            slot,
            class: DeviceType::new(class),
        }
    }
}

fn config_read_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = ((bus as u32) << 16)
        | ((slot as u32 & 0x1F) << 11)
        | ((func as u32 & 0b11) << 8)
        | (offset as u32 & 0xFC)
        | ENABLE;

    // SAFETY: We can read any of the 256 registers of the Configuration Space safely,
    // offset, func, slot and bus are constrained respectively to 256, 8, 32, 256
    // Reading a non-existant device is not an issue
    unsafe {
        port::outd(CONFIG_ADDRESS, address);
        port::ind(CONFIG_DATA)
    }
}

fn config_read_u16(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let value = config_read_u32(bus, slot, func, offset);
    if offset & 0b10 != 0 {
        (value >> 16) as u16
    } else {
        value as u16
    }
}

fn config_read_u8(bus: u8, slot: u8, func: u8, offset: u8) -> u8 {
    let value = config_read_u16(bus, slot, func, offset);
    if offset & 1 != 0 {
        (value >> 8) as u8
    } else {
        value as u8
    }
}

fn device_exists(bus: u8, slot: u8, function: u8) -> bool {
    config_read_u16(bus, slot, function, VENDOR_ID) != 0xFFFF
}

pub fn init() {
    crate::serial_println!("{:?}", discover_devices());
}

/// Discover the devices handled by the PCI host controller
/// Current implementation does not extend discovery to PCI-to-PCI brigdes
fn discover_devices() -> Vec<Device> {
    let mut devices = Vec::new();
    if is_multi_func(0, 0) {
        for function in 0..8 {
            if device_exists(0, 0, function) {
                discover_devices_in_bus(function, &mut devices);
            }
        }
    } else {
        discover_devices_in_bus(0, &mut devices)
    }

    devices
}

fn discover_devices_in_bus(bus: u8, devices: &mut Vec<Device>) {
    for slot in 0..32 {
        if device_exists(bus, slot, 0) {
            let class = config_read_u16(bus, slot, 0, CLASS);
            devices.push(Device::new(bus, slot, class));
            crate::serial_println!(
                "deviceID: {:x}, vendorID: {:x}, {:?}",
                config_read_u16(bus, slot, 0, DEVICE_ID),
                config_read_u16(bus, slot, 0, VENDOR_ID),
                DeviceType::new(class)
            )
        }
    }
}

fn is_multi_func(bus: u8, slot: u8) -> bool {
    config_read_u8(bus, slot, 0, HEADER_TYPE) & 0x80 != 0
}
