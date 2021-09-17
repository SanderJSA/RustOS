use super::port;
use crate::driver;
use alloc::vec::Vec;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;
const ENABLE: u32 = 1 << 31;

const VENDOR_ID: u8 = 0;
pub const COMMAND: u8 = 4;
const CLASS: u8 = 10;
const HEADER_TYPE: u8 = 14;
const BAR: u8 = 16;

pub fn init() {
    let devices = discover_devices();
    for device in devices.iter() {
        crate::serial_println!(
            "Device at ({},{}) {:?} {:x?}",
            device.bus,
            device.slot,
            device.class(),
            device.id(),
        );
    }
    driver::init(&devices);
}

fn discover_devices() -> Vec<Device> {
    let mut devices = Vec::new();
    for bus in 0..=255 {
        for slot in 0..32 {
            if let Some(device) = Device::try_new(bus, slot) {
                devices.push(device);
            }
        }
    }
    devices
}

#[derive(Debug, Clone, Copy)]
pub struct Device {
    pub bus: u8,
    pub slot: u8,
}

impl Device {
    /// Creates a PCI device at bus and slot address
    /// Returns None if device does not exist
    pub fn try_new(bus: u8, slot: u8) -> Option<Device> {
        let device = Device { bus, slot };
        match device.exists(Function::Zero) {
            true => Some(device),
            false => None,
        }
    }

    pub fn class(&self) -> DeviceClass {
        DeviceClass::new(self.read_u16(Function::Zero, CLASS))
    }

    pub fn id(&self) -> DeviceId {
        DeviceId::new(self.read_u32(Function::Zero, VENDOR_ID))
    }

    pub fn bar(&self, func: Function, register: u8) -> Option<Bar> {
        if register >= self.nb_bars(func) {
            return None;
        }

        let is_mmio = |bar| (bar & 1) == 0;
        let is_64bits = |bar| (bar & 0b110) == 0b100;

        let offset = BAR + register * 4;
        let bar = self.read_u32(func, offset) as usize;
        if is_mmio(bar) {
            let mut base = bar & !0xF;
            let prefetchable = bar & 0b1000 != 0;
            if is_64bits(bar) {
                base += (self.read_u32(func, offset + 4) as usize) << 32;
            }
            // SAFETY: bar register exists and gets restored after size probing
            let size;
            unsafe {
                self.write_u32(func, offset, !0);
                size = 1 << self.read_u32(func, offset).trailing_zeros();
                self.write_u32(func, offset, bar as u32);
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

    pub fn read_u32(&self, func: Function, offset: u8) -> u32 {
        let address = config_address(self.bus, self.slot, func, offset);
        // SAFETY: We can read any of the 256 registers of the Configuration Space safely,
        // offset, func, slot and bus are constrained respectively to 256, 8, 32, 256
        // Reading a non-existant device is not an issue
        unsafe {
            port::outd(CONFIG_ADDRESS, address);
            port::ind(CONFIG_DATA)
        }
    }

    pub fn read_u16(&self, func: Function, offset: u8) -> u16 {
        let value = self.read_u32(func, offset);
        if offset & 0b10 != 0 {
            (value >> 16) as u16
        } else {
            value as u16
        }
    }

    pub fn read_u8(&self, func: Function, offset: u8) -> u8 {
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
    pub unsafe fn write_u32(&self, func: Function, offset: u8, data: u32) {
        let address = config_address(self.bus, self.slot, func, offset);
        port::outd(CONFIG_ADDRESS, address);
        port::outd(CONFIG_DATA, data);
    }

    /// Write u16 to PCI device
    /// # Safety
    /// May cause side-effects
    pub unsafe fn write_u16(&self, func: Function, offset: u8, data: u16) {
        let address = config_address(self.bus, self.slot, func, offset);
        port::outd(CONFIG_ADDRESS, address);
        port::outw(CONFIG_DATA, data);
    }

    pub fn exists(&self, function: Function) -> bool {
        self.read_u16(function, VENDOR_ID) != 0xFFFF
    }

    fn nb_bars(&self, func: Function) -> u8 {
        const LEAF_DEVICE_TYPE: u8 = 0;
        const PCI_TO_PCI_TYPE: u8 = 1;
        const PCI_TO_CARDBUS_TYPE: u8 = 2;

        match self.read_u8(func, HEADER_TYPE) {
            LEAF_DEVICE_TYPE => 6,
            PCI_TO_PCI_TYPE => 2,
            PCI_TO_CARDBUS_TYPE => 0,
            _ => panic!("Unknown header type"),
        }
    }
}

fn config_address(bus: u8, slot: u8, func: Function, offset: u8) -> u32 {
    ((bus as u32) << 16)
        | ((slot as u32 & 0x1F) << 11)
        | ((func as u32 & 0b11) << 8)
        | (offset as u32 & 0xFC)
        | ENABLE
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

#[allow(dead_code)]
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

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Function {
    Zero = 0,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}
