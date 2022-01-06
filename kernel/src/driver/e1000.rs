use core::default::Default;
use core::mem;

/// e1000 driver based on
/// https://www.intel.com/content/dam/www/public/us/en/documents/manuals/pcie-gbe-controllers-open-source-manual.pdf
use crate::arch::pci::*;
use crate::memory_manager::{self, EntryFlag, PAGE_SIZE};

pub const DEVICE_TYPE: DeviceClass = DeviceClass::EthernetController;

const CTRL: u16 = 0;
const EEC: u16 = 0x10;
const EERD: u16 = 0x14;
const IMC: u16 = 0xD8;
const ICR: u16 = 0xC0;

// RX Registess
const RX_DESC_LO: u16 = 0x2800;
const RX_DESC_HI: u16 = 0x2804;
const RX_DESC_LEN: u16 = 0x2808;
const RX_DESC_HEAD: u16 = 0x2810;
const RX_DESC_TAIL: u16 = 0x2818;
const RX_CTRL: u16 = 0x0100;

const RESET: u32 = 0x4000000;
const EEPROM_PRESENT: u32 = 1 << 8;

const NB_RX_DESC: usize = 32;
const NB_TX_DESC: usize = 8;

pub fn init(device: &Device) {
    let e1000 = E1000::new(device);
    e1000.reset();
    e1000.init();
}

struct E1000 {
    device: Device,
    pub mmio: usize,
    rx_descs: [RxDesc; NB_RX_DESC],
    tx_descs: [TxDesc; NB_TX_DESC],
}

impl E1000 {
    pub fn new(device: &Device) -> E1000 {
        if let Some(Bar::MMIO { base, size, .. }) = device.bar(Function::Zero, 0) {
            crate::memory_manager::mmio_map(base, size);
            E1000 {
                device: *device,
                mmio: base,
                rx_descs: Default::default(),
                tx_descs: Default::default(),
            }
        } else {
            panic!("Unexpected BAR form");
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.device.write_u16(Function::Zero, COMMAND, 0b111);
            // Mask interrupts and clear them
            self.mmio_outd(IMC, 0xFFFFFFFF);
            self.mmio_ind(ICR);
            self.mmio_outd(CTRL, RESET);
        }
    }

    pub fn init(&self) {
        crate::serial_println!("EEPROM present: {}", self.is_eeprom_present());
        self.get_mac();
        self.set_link_up();
        self.rx_init();
    }

    fn get_mac(&self) {
        let val = self.read_eeprom(0);
        crate::serial_print!("MAC address: {}:{}", val as u8, (val >> 8) as u8);
        let val = self.read_eeprom(1);
        crate::serial_print!(":{}:{}", val as u8, (val >> 8) as u8);
        let val = self.read_eeprom(2);
        crate::serial_println!(":{}:{}", val as u8, (val >> 8) as u8);
    }

    /// Read u32 from device
    /// # Safety
    /// Read must be within device boundary, may cause side effect
    unsafe fn mmio_ind(&self, reg: u16) -> u32 {
        let addr = (self.mmio + reg as usize) as *const u32;
        addr.read_volatile()
    }

    /// Write u32 to device
    /// # Safety
    /// Read must be within device boundary, may cause side effect
    unsafe fn mmio_outd(&self, reg: u16, value: u32) {
        let addr = (self.mmio + reg as usize) as *mut u32;
        addr.write_volatile(value);
    }

    fn is_eeprom_present(&self) -> bool {
        // SAFETY: EEC is a valid register
        let val = unsafe { self.mmio_ind(EEC) };
        val & EEPROM_PRESENT != 0
    }

    fn read_eeprom(&self, addr: u16) -> u16 {
        const START: u32 = 1;
        const DONE: u32 = 1 << 4;

        // SAFETY: EERD is within boundaries and we only retrieve data from EEPROM
        unsafe {
            self.mmio_outd(EERD, START | ((addr as u32 & 0x3FFF) << 8));
        }
        loop {
            // SAFETY: EERD is within boundaries
            let read = unsafe { self.mmio_ind(EERD) };
            if read & DONE != 0 {
                return (read >> 16) as u16;
            }
        }
    }

    fn set_link_up(&self) {
        const SLU: u32 = 1 << 6;
        // SAFETY: CTRL exists
        unsafe {
            let ctrl = self.mmio_ind(CTRL);
            self.mmio_outd(CTRL, ctrl | SLU);
        }
    }

    fn rx_init(&self) {
        // RX_CTL Values
        const EN: u32 = 1 << 1; // Enable receiver
        const SBP: u32 = 1 << 2; // Store bad packets
        const UPE: u32 = 1 << 3; // Unicast Promiscuous Enabled
        const MPE: u32 = 1 << 4; // Multicast Promiscuous Enabled
        const BAM: u32 = 1 << 15; // Broadcast Accept Mode
        const BSIZE: u32 = 11 << 16; // Set buffer size to 4096
        const BSEX: u32 = 1 << 25; //
        const SECRC: u32 = 1 << 26; // Strip Ethernet CRC from incoming packet

        let descs = self.rx_descs.as_ptr() as u64;
        // SAFETY: Pointer points to valid rx descriptors ring
        // However that promise is brittle at the moment, if the structure ever moves that promise
        // is broken
        unsafe {
            self.mmio_outd(RX_DESC_LO, descs as u32);
            self.mmio_outd(RX_DESC_HI, (descs >> 32) as u32);
            self.mmio_outd(RX_DESC_LEN, (mem::size_of::<RxDesc>() * NB_RX_DESC) as u32);
            self.mmio_outd(RX_DESC_HEAD, 0);
            self.mmio_outd(RX_DESC_TAIL, NB_RX_DESC as u32 - 1);
            self.mmio_outd(RX_CTRL, EN | SBP | UPE | MPE | BAM | SECRC | BSEX | BSIZE);
        }
    }
}

#[repr(C)]
struct RxDesc {
    /// Must contain the physical address
    addr: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    tag: u16,
}

impl Default for RxDesc {
    fn default() -> Self {
        RxDesc {
            addr: memory_manager::mmap(None, EntryFlag::Writable as u64) as u64,
            length: PAGE_SIZE as u16,
            checksum: 0,
            status: 0,
            errors: 0,
            tag: 0,
        }
    }
}

#[repr(C)]
struct TxDesc {
    /// Must contain the physical address
    addr: u64,
    length: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

impl Default for TxDesc {
    fn default() -> Self {
        TxDesc {
            addr: memory_manager::mmap(None, EntryFlag::Writable as u64) as u64,
            length: PAGE_SIZE as u16,
            cso: 0,
            cmd: 0,
            status: 0,
            css: 0,
            special: 0,
        }
    }
}
