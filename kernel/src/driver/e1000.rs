use core::default::Default;
use core::{mem, ptr};

use alloc::vec::Vec;

/// e1000 driver based on
/// https://www.intel.com/content/dam/www/public/us/en/documents/manuals/pcie-gbe-controllers-open-source-manual.pdf
use crate::arch::pci::*;
use crate::arch::pic::PICS;
use crate::driver::ETHERNET_DEVICE;
use crate::memory_manager::{self, EntryFlag, PAGE_SIZE};

pub const DEVICE_TYPE: DeviceClass = DeviceClass::EthernetController;

// Registers
const CTRL: u16 = 0;
const STATUS: u16 = 8;
const EEC: u16 = 0x10;
const EERD: u16 = 0x14;
const IMC: u16 = 0xD8;
const ICR: u16 = 0xC0;
const IMS: u16 = 0xD0;
const MULTICAST_TABLE: u16 = 0x5200;

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
const TIPG: u16 = 0x0410;

const RESET: u32 = 0x4000000;
const EEPROM_PRESENT: u32 = 1 << 8;

const NB_RX_DESC: usize = 16;
const NB_TX_DESC: usize = 8;

const MMAP_FLAGS: u64 =
    EntryFlag::Writable as u64 + EntryFlag::WriteThrough as u64 + EntryFlag::NoCache as u64;

pub fn init(device: &Device) {
    unsafe {
        ETHERNET_DEVICE = Some(E1000::new(device));
        if let Some(e1000) = &mut ETHERNET_DEVICE {
            e1000.reset();
            e1000.init();
            crate::serial_println!("Link status is up: {}", e1000.mmio_ind(STATUS) & 3 != 0);
        }
    }
}

pub struct E1000 {
    device: Device,
    pub mmio: usize,
    rx_descs: &'static mut [RxDesc; NB_RX_DESC],
    tx_descs: &'static mut [TxDesc; NB_TX_DESC],
    rx_cur: usize,
    tx_cur: usize,
    mac_address: [u8; 6],
}

impl E1000 {
    pub fn new(device: &Device) -> E1000 {
        if let Some(Bar::MMIO { base, size, .. }) = device.bar(Function::Zero, 0) {
            crate::memory_manager::mmio_map(base, size);
            assert!(mem::size_of::<[RxDesc; NB_RX_DESC]>() <= PAGE_SIZE);
            assert!(mem::size_of::<[TxDesc; NB_TX_DESC]>() <= PAGE_SIZE);
            // SAFETY: Descriptors fit within allocated memory
            // Descriptors need to point to the same memory region even if the structure is moved
            // As it is the memory region used by hardware
            let rx_descs = unsafe { &mut *(memory_manager::mmap(None, MMAP_FLAGS) as *mut _) };
            let tx_descs = unsafe { &mut *(memory_manager::mmap(None, MMAP_FLAGS) as *mut _) };
            E1000 {
                device: *device,
                mmio: base,
                rx_descs,
                tx_descs,
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

        set_interrupt_enabled(false);
        let desc = &mut self.tx_descs[self.tx_cur] as &mut TxDesc;
        desc.addr = data.as_ptr() as u64;
        desc.length = data.len() as u16;
        desc.cmd = EOP | IFCS | RS;
        desc.status = 0;

        let next_tx = (self.tx_cur + 1) % NB_TX_DESC;
        unsafe {
            self.mmio_outd(TX_DESC_TAIL, next_tx as u32);
            while (&self.tx_descs[self.tx_cur] as *const TxDesc)
                .read_volatile()
                .status
                == 0
            {}
        }
        self.tx_cur = next_tx;
        set_interrupt_enabled(false);
        crate::serial_println!("done");
    }

    pub fn create_ethernet_frame(&self, dst: &[u8; 6], ethertype: u16, payload: &[u8]) -> Vec<u8> {
        let mut frame: Vec<u8> = dst.to_vec();
        frame.extend(self.mac_address);
        frame.push((ethertype >> 8) as u8);
        frame.push(ethertype as u8);
        frame.extend(payload);

        frame
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
        // Empty Multicast table array
        for i in 0..128 {
            // SAFETY: MULTICAST_TABLE array is a valid memory region
            unsafe {
                self.mmio_outd(MULTICAST_TABLE + i * 4, 0);
            }
        }
        self.enable_interrupts();
        self.rx_init();
        self.tx_init();
    }

    fn set_mac(&mut self) {
        crate::serial_print!("MAC address ");
        for i in 0..3 {
            let val = self.read_eeprom(i as u16);
            self.mac_address[i] = val as u8;
            self.mac_address[i + 1] = (val >> 8) as u8;
            crate::serial_print!(":{:X}:{:X}", self.mac_address[i], self.mac_address[i + 1]);
        }
        crate::serial_println!("");
    }

    /// Read u32 from device
    /// # Safety
    /// Read must be within device boundary, may cause side effect
    pub unsafe fn mmio_ind(&self, reg: u16) -> u32 {
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

    fn rx_init(&mut self) {
        // RX_CTL Values
        const EN: u32 = 1 << 1; // Enable receiver
        const SBP: u32 = 1 << 2; // Store bad packets
        const UPE: u32 = 1 << 3; // Unicast Promiscuous Enabled
        const MPE: u32 = 1 << 4; // Multicast Promiscuous Enabled
        const BAM: u32 = 1 << 15; // Broadcast Accept Mode
        const BSIZE: u32 = 11 << 16; // Set buffer size to 4096
        const BSEX: u32 = 1 << 25; //
        const SECRC: u32 = 1 << 26; // Strip Ethernet CRC from incoming packet

        for descs in self.rx_descs.iter_mut() {
            *descs = RxDesc::default();
        }

        let descs = self.rx_descs.as_ptr() as u64;
        // SAFETY: Pointer points to valid rx descriptors ring
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
        unsafe {
            self.mmio_outd(TX_DESC_LO, descs as u32);
            self.mmio_outd(TX_DESC_HI, (descs >> 32) as u32);
            self.mmio_outd(TX_DESC_LEN, (mem::size_of::<TxDesc>() * NB_TX_DESC) as u32);
            self.mmio_outd(TX_DESC_HEAD, 0);
            self.mmio_outd(TX_DESC_TAIL, 0);
            self.mmio_outd(TX_CTRL, EN | PSP | CT | COLD | RTLC);

            // The recommended TIPG value to achieve 802.3 compliant minimum transmit IPG values
            // in full and half duplex is 00702008h.
            self.mmio_outd(TIPG, 0x00702008);
        }
    }

    fn enable_interrupts(&self) {
        const TXDW: u32 = 1; // Transmit Descriptor Written Back
        const TXQE: u32 = 1 << 1; // Transmit Queue Empty
        const LSC: u32 = 1 << 2; // Link Status Change.
        const RXSEQ: u32 = 1 << 3; // Receive Sequence Error.
        const RXDMT0: u32 = 1 << 4; // Receive Descriptor Minimum Threshold hit.
        const RXO: u32 = 1 << 6; // Receiver Overrun. Sets on Receive Data FIFO Overrun
        const RXT0: u32 = 1 << 7; // Receiver Timer Interrupt.
        unsafe {
            self.mmio_outd(IMS, TXDW | TXQE | LSC | RXSEQ | RXDMT0 | RXO | RXT0);
            self.mmio_ind(0xC0); // Clear previous interrupts
        }
    }
}

fn set_interrupt_enabled(is_enabled: bool) {
    PICS.obtain().set_mask(match is_enabled {
        true => 1 << 11,
        false => 0,
    });
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
            addr: memory_manager::mmap(None, MMAP_FLAGS) as u64,
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
