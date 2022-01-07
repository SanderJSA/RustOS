use core::default::Default;
use core::mem;

use alloc::vec::Vec;

/// e1000 driver based on
/// https://www.intel.com/content/dam/www/public/us/en/documents/manuals/pcie-gbe-controllers-open-source-manual.pdf
use crate::arch::pci::*;
use crate::memory_manager::{self, EntryFlag};

pub const DEVICE_TYPE: DeviceClass = DeviceClass::EthernetController;

// Registers
const CTRL: u16 = 0;
const EEC: u16 = 0x10;
const EERD: u16 = 0x14;
const IMC: u16 = 0xD8;
const ICR: u16 = 0xC0;
const IMS: u16 = 0xD0;

// RX Registers
const RX_DESC_LO: u16 = 0x2800;
const RX_DESC_HI: u16 = 0x2804;
const RX_DESC_LEN: u16 = 0x2808;
const RX_DESC_HEAD: u16 = 0x2810;
const RX_DESC_TAIL: u16 = 0x2818;
const RX_CTRL: u16 = 0x0100;

// TX Registers
const TX_DESC_LO: u16 = 0x3800;
const TX_DESC_HI: u16 = 0x3804;
const TX_DESC_LEN: u16 = 0x3808;
const TX_DESC_HEAD: u16 = 0x3810;
const TX_DESC_TAIL: u16 = 0x3818;
const TX_CTRL: u16 = 0x0400;

const RESET: u32 = 0x4000000;
const EEPROM_PRESENT: u32 = 1 << 8;

const NB_RX_DESC: usize = 32;
const NB_TX_DESC: usize = 8;

pub fn init(device: &Device) -> E1000 {
    let mut e1000 = E1000::new(device);
    e1000.reset();
    crate::serial_println!("reseted e1000");
    e1000.init();
    crate::serial_println!("e1000 init");
    let packet = e1000.create_ethernet_frame(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], &[0, 1, 2, 3]);
    crate::serial_println!("sending packet...");
    e1000.send_packet(&packet);
    e1000
}

pub struct E1000 {
    device: Device,
    pub mmio: usize,
    rx_descs: [RxDesc; NB_RX_DESC],
    tx_descs: [TxDesc; NB_TX_DESC],
    rx_cur: usize,
    tx_cur: usize,
    mac_address: [u8; 6],
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
                rx_cur: 0,
                tx_cur: 0,
                mac_address: Default::default(),
            }
        } else {
            panic!("Unexpected BAR form");
        }
    }

    pub fn send_packet(&mut self, data: &[u8]) {
        const EOP: u8 = 1; // End of packet
        const IFCS: u8 = 1 << 1; // Insert FCS
        const RS: u8 = 1 << 3; // Report status

        let desc = &mut self.tx_descs[self.tx_cur];
        desc.addr = data.as_ptr() as u64;
        desc.length = data.len() as u16;
        desc.cmd = EOP | IFCS | RS;
        desc.status = 0;

        unsafe {
            self.mmio_outd(TX_DESC_TAIL, self.tx_cur as u32);
            while (&self.tx_descs[self.tx_cur] as *const TxDesc)
                .read_volatile()
                .status
                != 0
            {
                crate::serial_println!("sending")
            }
        }
        self.tx_cur = (self.tx_cur + 1) % NB_TX_DESC;
        crate::serial_println!("done");
    }

    pub fn create_ethernet_frame(&self, dst: &[u8; 6], payload: &[u8]) -> Vec<u8> {
        let preamble_byte: u8 = 0b10101010;
        let start_of_frame_delim: u8 = 0b10101011;

        let mut frame = alloc::vec![preamble_byte; 7];
        frame.push(start_of_frame_delim);
        frame.extend(dst);
        frame.extend(self.mac_address);
        frame.push(payload.len() as u8);
        frame.push((payload.len() >> 8) as u8);
        frame.extend(payload);

        frame

        //frame.push(0x0806u16 as u8);
        //frame.push((0x0806 >> 8) as u8);
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

    pub fn init(&mut self) {
        crate::serial_println!("EEPROM present: {}", self.is_eeprom_present());
        self.set_mac();
        self.set_link_up();
        for i in 0..128 {
            unsafe {
                self.mmio_outd(0x5200 + i * 4, 0);
            }
        }
        self.enable_interrupts();
        self.rx_init();
        self.tx_init();
        crate::serial_println!("int: {}", self.device.read_u8(Function::Zero, INT_LINE));
        unsafe {
            self.device.write_u8(Function::Zero, INT_LINE, 11);
        }
        crate::serial_println!("int: {}", self.device.read_u8(Function::Zero, INT_LINE));
    }

    fn set_mac(&mut self) {
        let val = self.read_eeprom(0);
        self.mac_address[0] = val as u8;
        self.mac_address[1] = (val >> 8) as u8;
        let val = self.read_eeprom(1);
        self.mac_address[2] = val as u8;
        self.mac_address[3] = (val >> 8) as u8;
        let val = self.read_eeprom(2);
        self.mac_address[4] = val as u8;
        self.mac_address[5] = (val >> 8) as u8;
        crate::serial_print!(
            "MAC address: {}:{}:{}:{}:{}:{}",
            self.mac_address[0],
            self.mac_address[1],
            self.mac_address[2],
            self.mac_address[3],
            self.mac_address[4],
            self.mac_address[5]
        );
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

    fn tx_init(&self) {
        // TX_CTL Values
        const EN: u32 = 1 << 1; // Enable transmitter
        const PSP: u32 = 1 << 3; // Pad short packets
        const CT: u32 = 0x0F << 4; // Number of attempts at retransmitting packet
        const COLD: u32 = 0x3F << 12; // Collision distance
        const RTLC: u32 = 1 << 24; // Re-transmit on Late Collision

        let descs = self.tx_descs.as_ptr() as u64;
        // SAFETY: Pointer points to valid rx descriptors ring
        // However that promise is brittle at the moment, if the structure ever moves that promise
        // is broken
        unsafe {
            self.mmio_outd(TX_DESC_LO, descs as u32);
            self.mmio_outd(TX_DESC_HI, (descs >> 32) as u32);
            self.mmio_outd(TX_DESC_LEN, (mem::size_of::<TxDesc>() * NB_RX_DESC) as u32);
            self.mmio_outd(TX_DESC_HEAD, 0);
            self.mmio_outd(TX_DESC_TAIL, 0);
            self.mmio_outd(TX_CTRL, EN | PSP | CT | COLD | RTLC);
        }
    }

    fn enable_interrupts(&self) {
        // Enable all interupts except parity errors and TxDesc writeback and TxQueue_empty
        unsafe {
            self.mmio_outd(IMS, 1 | 1 << 2 | 1 << 3 | 1 << 4 | 1 << 7);
            //self.mmio_outd(IMS, 0x1F6DC);
            //self.mmio_outd(IMS, 0x1F6DF);
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
            length: 0,
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
            addr: 0,
            length: 0,
            cso: 0,
            cmd: 0,
            status: 1,
            css: 0,
            special: 0,
        }
    }
}
