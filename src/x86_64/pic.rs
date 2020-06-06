use x86_64::port::{inb, outb};

const PIC_INIT: u8 = 0x11;
const END_OF_INTERRUPT: u8 = 0x20;
const MODE_8086: u8 = 0x01;

struct Pic {
    offset: u8,
    command_port: u16,
    data_port: u16,
}

impl Pic {
    fn handles_interrupt(&self, interupt_id: u8) -> bool {
        self.offset <= interupt_id && interupt_id < self.offset + 8
    }

    fn end_of_interrupt(&self) {
        outb(self.command_port, END_OF_INTERRUPT);
    }
}

pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const fn new(offset1: u8, offset2: u8) -> ChainedPics {
        ChainedPics {
            pics: [
                Pic {
                    offset: offset1,
                    command_port: 0x20,
                    data_port: 0x21,
                },
                Pic {
                    offset: offset2,
                    command_port: 0xA0,
                    data_port: 0xA1,
                },
            ]
        }
    }

    pub fn initialize(&mut self) {
        let io_wait = || {outb(0x80, 0)};

        // Get PIC's masks
        let mask_pic1 = inb(self.pics[0].data_port);
        let mask_pic2 = inb(self.pics[1].data_port);

        // Initialize PIC with correct args
        outb(self.pics[0].command_port, PIC_INIT);
        io_wait();
        outb(self.pics[1].command_port, PIC_INIT);
        io_wait();

        outb(self.pics[0].data_port, self.pics[0].offset);
        io_wait();
        outb(self.pics[1].data_port, self.pics[1].offset);
        io_wait();

        outb(self.pics[0].data_port, 4);
        io_wait();
        outb(self.pics[1].data_port, 2);
        io_wait();

        outb(self.pics[0].data_port, MODE_8086);
        io_wait();
        outb(self.pics[1].data_port, MODE_8086);
        io_wait();


        outb(self.pics[0].data_port, mask_pic1);
        outb(self.pics[1].data_port, mask_pic2);
    }

    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(interrupt_id))
    }

    pub fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.handles_interrupt(interrupt_id) {
            if self.pics[1].handles_interrupt(interrupt_id) {
                self.pics[1].end_of_interrupt();
            }
            self.pics[0].end_of_interrupt();
        }
    }
}