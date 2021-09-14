use crate::arch::paging::tables::EntryFlag;
use crate::arch::pci::*;
use crate::arch::port;

pub const DEVICE_TYPE: DeviceClass = DeviceClass::EthernetController;

const RESET: u32 = 0x4000000;

const EEPROM: usize = 0x00014;
const EECD: usize = 0x00010;

const EE_PRES: u32 = 0x8;

/// Registers
const CTRL: u16 = 0;
const RCTRL: u16 = 0x0100;
const IMC: u16 = 0x000D8;
const ICR: u16 = 0x000C0;

pub fn init(device: &Device) {
    let e1000 = E1000::new(device);
    e1000.reset();
    let base = e1000.mmio;

    crate::serial_println!("EEPROM PRESENT: {}", detect_eeprom(base));

    /*
    unsafe {
        crate::print!("value: {:x}\nread:", *(base as *const u32));

        for i in 0..6 {
            crate::serial_println!("{}", *((base + 0x5400 + i) as *mut u8));
        }
    }
    */
    crate::println!("{}", read_eeprom(base as usize, 0x00));
    crate::println!("{}", read_eeprom(base as usize, 0x01));
    crate::println!("{}", read_eeprom(base as usize, 0x02));
}

fn read_eeprom(mmio_addr: usize, addr: u8) -> u16 {
    let eecd = (mmio_addr + EECD) as *mut u32;
    unsafe {
        if *eecd & EE_PRES == 0 {
            panic!("EEPROM not present");
        }
    }

    let eerd = (mmio_addr + EEPROM) as *mut u32;
    unsafe {
        *eerd = 1 | ((addr as u32) << 8);
        loop {
            let tmp = *eerd & (1 << 4);
            if tmp != 0 {
                return (tmp >> 16) as u16;
            }
        }
    }
}

fn detect_eeprom(mmio_addr: usize) -> bool {
    let eeprom = 49152 + EEPROM;

    unsafe {
        port::outd(eeprom as u16, 1);
        for _ in 0..1000 {
            if port::ind(eeprom as u16) & 0x10 != 0 {
                return true;
            }
        }
    }
    false
}

fn eeprom_read(mmio_addr: usize, addr: u8) -> u32 {
    let reg = (mmio_addr + EEPROM) as *mut u32;

    unsafe {
        reg.write_volatile(1 | (addr as u32) << 2);
        loop {
            let tmp = *reg;
            if tmp & 0b10 != 0 {
                return tmp >> 16;
            }
        }
    }
}

struct E1000 {
    device: Device,
    pub mmio: usize,
    io: u16,
}

impl E1000 {
    pub fn new(device: &Device) -> E1000 {
        if let (Some(Bar::MMIO { base, .. }), Some(Bar::IO { port })) =
            (device.bar(0), device.bar(1))
        {
            crate::memory_manager::mmap(
                Some(base as usize),
                EntryFlag::Writable as u64
                    + EntryFlag::WriteThrough as u64
                    + EntryFlag::NoCache as u64,
            );
            E1000 {
                mmio: base as usize,
                io: port as u16,
                device: *device,
            }
        } else {
            panic!("Unexpected BAR form");
        }
    }

    pub fn reset(&self) {
        unsafe {
            let device = self.device;
            config_write_u16(device.bus, device.slot, 0, COMMAND, 0b111);
            self.io_outd(IMC, 0xFFFFFFFF);
            self.io_ind(ICR);
            self.io_outd(CTRL, RESET | (1 << 31));
            crate::serial_println!("{}", self.io_ind(CTRL));
        }
    }

    unsafe fn io_ind(&self, reg: u16) -> u32 {
        let addr = (self.mmio + reg as usize) as *const u32;
        *addr
        /*
        port::outd(self.io, reg as u32);
        port::ind(self.io + 4)
        */
    }

    unsafe fn io_outd(&self, reg: u16, value: u32) {
        let addr = (self.mmio + reg as usize) as *mut u32;
        *addr = value;
        /*
        port::outd(self.io, reg as u32);
        port::outd(self.io + 4, value);
        */
    }
}
