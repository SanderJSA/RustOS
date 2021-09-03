use super::port::outb;
use crate::utils::lazy_static::LazyStatic;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

const MASTER_PIC: u16 = 0x20;
const SLAVE_PIC: u16 = 0xA0;
const DATA_OFFSET: u16 = 1;

const IRQS_PER_PIC: u8 = 8;
const SLAVE_IRQ: u8 = 2;
const ICW1: u8 = 0b00010001;
const ICW4: u8 = 0b00000001;
const END_OF_INTERRUPT: u8 = 0x20;

pub static PICS: LazyStatic<ChainedPics> = LazyStatic::new(ChainedPics::new);

pub fn init() {
    PICS.obtain();
}

struct Pic {
    offset: u8,
    port: u16,
}

impl Pic {
    pub unsafe fn new(offset: u8, port: u16, is_master: bool) -> Pic {
        outb(port, ICW1);
        outb(port + DATA_OFFSET, offset);
        match is_master {
            true => outb(port + DATA_OFFSET, 1 << SLAVE_IRQ),
            false => outb(port + DATA_OFFSET, SLAVE_IRQ),
        };
        outb(port + DATA_OFFSET, ICW4);

        Pic { offset, port }
    }

    pub fn handles_interrupt(&self, interupt_id: u8) -> bool {
        (self.offset..self.offset + IRQS_PER_PIC).contains(&interupt_id)
    }

    pub unsafe fn end_of_interrupt(&self) {
        outb(self.port, END_OF_INTERRUPT);
    }
}

pub struct ChainedPics {
    master: Pic,
    slave: Pic,
}

impl ChainedPics {
    pub fn new() -> ChainedPics {
        // SAFETY: MASTER_PIC and SLAVE_PIC are both valid PIC ports
        unsafe {
            ChainedPics {
                master: Pic::new(PIC_1_OFFSET, MASTER_PIC, true),
                slave: Pic::new(PIC_2_OFFSET, SLAVE_PIC, false),
            }
        }
    }

    pub fn notify_end_of_interrupt(&self, interrupt_id: u8) {
        // SAFETY: Master and slave PIC are both initialized
        unsafe {
            self.master.end_of_interrupt();
            if self.slave.handles_interrupt(interrupt_id) {
                self.slave.end_of_interrupt();
            }
        }
    }

    pub fn end_all_interrupts(&self) {
        // SAFETY: Master and slave PIC are both initialized
        unsafe {
            self.master.end_of_interrupt();
            self.slave.end_of_interrupt();
        }
    }
}

impl Default for ChainedPics {
    fn default() -> Self {
        Self::new()
    }
}
