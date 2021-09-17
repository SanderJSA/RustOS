/// e1000 driver based on
/// https://www.intel.com/content/dam/www/public/us/en/documents/manuals/pcie-gbe-controllers-open-source-manual.pdf
use crate::arch::paging::tables::EntryFlag;
use crate::arch::pci::*;

pub const DEVICE_TYPE: DeviceClass = DeviceClass::EthernetController;

const CTRL: u16 = 0;
const EEC: u16 = 0x10;
//const EERD: u16 = 0x14;
const IMC: u16 = 0xD8;
const ICR: u16 = 0xC0;

const RESET: u32 = 0x4000000;
const EEPROM_PRESENT: u32 = 1 << 8;

pub fn init(device: &Device) {
    let e1000 = E1000::new(device);
    e1000.reset();
    e1000.init();
}

struct E1000 {
    device: Device,
    pub mmio: usize,
}

impl E1000 {
    pub fn new(device: &Device) -> E1000 {
        if let Some(Bar::MMIO { base, .. }) = device.bar(0) {
            crate::memory_manager::mmap(
                Some(base as usize),
                EntryFlag::Writable as u64
                    + EntryFlag::WriteThrough as u64
                    + EntryFlag::NoCache as u64,
            );
            E1000 {
                device: *device,
                mmio: base as usize,
            }
        } else {
            panic!("Unexpected BAR form");
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.device.write_u16(0, COMMAND, 0b111);
            // Mask interrupts and clear them
            self.mmio_outd(IMC, 0xFFFFFFFF);
            self.mmio_ind(ICR);
            self.mmio_outd(CTRL, RESET);
        }
    }

    pub fn init(&self) {
        crate::serial_println!("EEPROM present: {}", self.is_eeprom_present());
    }

    unsafe fn mmio_ind(&self, reg: u16) -> u32 {
        let addr = (self.mmio + reg as usize) as *const u32;
        addr.read_volatile()
    }

    unsafe fn mmio_outd(&self, reg: u16, value: u32) {
        let addr = (self.mmio + reg as usize) as *mut u32;
        addr.write_volatile(value);
    }

    fn is_eeprom_present(&self) -> bool {
        // SAFETY: EEC is a valid register
        let val = unsafe { self.mmio_ind(EEC) };
        val & EEPROM_PRESENT != 0
    }
}
