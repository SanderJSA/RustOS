use super::port;
use crate::driver;
use alloc::vec::Vec;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;
const ENABLE: u32 = 1 << 31;

const VENDOR_ID: u8 = 0;
pub const COMMAND: u8 = 4;
const CLASS: u8 = 10;
const BAR: u8 = 16;

#[derive(Debug, Clone, Copy)]
pub struct Device {
    pub bus: u8,
    pub slot: u8,
}

impl Device {
    /// Creates a PCI device at bus and slot address
    /// Returns None if device does not exist
    pub fn new(bus: u8, slot: u8) -> Option<Device> {
        let device = Device { bus, slot };
        match device.exists(0) {
            true => Some(device),
            false => None,
        }
    }

    pub fn class(&self) -> DeviceClass {
        DeviceClass::new(self.read_u16(0, CLASS))
    }

    pub fn id(&self) -> DeviceId {
        DeviceId::new(self.read_u32(0, VENDOR_ID))
    }

    pub fn bar(&self, register: u8) -> Option<Bar> {
        if register > 5 {
            return None;
        }

        let is_mmio = |bar| (bar & 1) == 0;
        let is_64bits = |bar| (bar & 0b110) == 0b100;

        let offset = BAR + register * 4;
        let bar = self.read_u32(0, offset) as usize;
        if is_mmio(bar) {
            let mut base = bar & !0xF;
            let prefetchable = bar & 0b1000 != 0;
            if is_64bits(bar) {
                base += (self.read_u32(0, offset + 4) as usize) << 32;
            }
            // SAFETY: bar register exists and gets restored after size probing
            let size;
            unsafe {
                self.write_u32(0, offset, !0);
                size = 1 << self.read_u32(0, offset).trailing_zeros();
                self.write_u32(0, offset, bar as u32);
            }
            Some(Bar::MMIO {
                base,
                size,
                prefetchable,
            })
        } else {
            Some(Bar::IO {
                port: bar as u32 & !0b11,
            })
        }
    }

    pub fn read_u32(&self, func: u8, offset: u8) -> u32 {
        let address = ((self.bus as u32) << 16)
            | ((self.slot as u32 & 0x1F) << 11)
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

    pub fn read_u16(&self, func: u8, offset: u8) -> u16 {
        let value = self.read_u32(func, offset);
        if offset & 0b10 != 0 {
            (value >> 16) as u16
        } else {
            value as u16
        }
    }

    pub fn read_u8(&self, func: u8, offset: u8) -> u8 {
        let value = self.read_u16(func, offset);
        if offset & 1 != 0 {
            (value >> 8) as u8
        } else {
            value as u8
        }
    }

    /// Write u32 to PCI device
    /// # Safety
    /// May cause side-effects
    pub unsafe fn write_u32(&self, func: u8, offset: u8, data: u32) {
        let address = ((self.bus as u32) << 16)
            | ((self.slot as u32 & 0x1F) << 11)
            | ((func as u32 & 0b11) << 8)
            | (offset as u32 & 0xFC)
            | ENABLE;

        port::outd(CONFIG_ADDRESS, address);
        port::outd(CONFIG_DATA, data);
    }

    /// Write u16 to PCI device
    /// # Safety
    /// May cause side-effects
    pub unsafe fn write_u16(&self, func: u8, offset: u8, data: u16) {
        let address = ((self.bus as u32) << 16)
            | ((self.slot as u32 & 0x1F) << 11)
            | ((func as u32 & 0b11) << 8)
            | (offset as u32 & 0xFC)
            | ENABLE;

        port::outd(CONFIG_ADDRESS, address);
        port::outw(CONFIG_DATA, data);
    }

    fn exists(&self, function: u8) -> bool {
        self.read_u16(function, VENDOR_ID) != 0xFFFF
    }
}

pub fn init() {
    let devices = discover_devices();
    for device in devices.iter() {
        crate::serial_println!(
            "Device at ({},{}) {:?} {:x?}, bar0: {:?}",
            device.bus,
            device.slot,
            device.class(),
            device.id(),
            device.bar(0).unwrap()
        );
    }
    driver::init(&devices);
}

fn discover_devices() -> Vec<Device> {
    let mut devices = Vec::new();
    for bus in 0..=255 {
        for slot in 0..32 {
            if let Some(device) = Device::new(bus, slot) {
                devices.push(device);
            }
        }
    }
    devices
}

#[derive(Debug)]
pub enum Bar {
    MMIO {
        base: usize,
        size: usize,
        prefetchable: bool,
    },
    IO {
        port: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    EthernetController,
    VgaCompatibleController,
    HostBridge,
    IsaBridge,
    Unknown(u16),
}

impl DeviceClass {
    pub fn new(class: u16) -> DeviceClass {
        match class {
            0x02_00 => DeviceClass::EthernetController,
            0x03_00 => DeviceClass::VgaCompatibleController,
            0x06_00 => DeviceClass::HostBridge,
            0x06_01 => DeviceClass::IsaBridge,
            _ => DeviceClass::Unknown(class),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceId {
    vendor_id: u16,
    device_id: u16,
}

impl DeviceId {
    pub fn new(device_id: u32) -> DeviceId {
        DeviceId {
            vendor_id: device_id as u16,
            device_id: (device_id >> 16) as u16,
        }
    }
}
