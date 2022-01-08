use super::gdt::KERNEL_CODE_SEG;
use super::pic::{PICS, PIC_1_OFFSET};
use super::port;
use crate::driver::ps2_keyboard;
use crate::println;
use core::mem::{self, MaybeUninit};

const MAX_ENTRIES: usize = 256;

const KEYBOARD_PORT: u16 = 0x60;

#[derive(Debug)]
#[repr(C)]
struct InterruptFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptFrame, _error_code: u64) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT ERR:{}\n{:#?}",
        _error_code, stack_frame
    );
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptFrame) {
    PICS.obtain()
        .notify_end_of_interrupt(InterruptIndex::Timer as u8);
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptFrame) {
    // SAFETY: KEYBOARD_PORT exists and has a value
    let scan_code = unsafe { port::inb(KEYBOARD_PORT) };
    ps2_keyboard::update_stdin(scan_code);
    PICS.obtain()
        .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
}
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptFrame, error_code: u64) {
    let address: usize;
    // SAFETY: cr2 contains invalid_access when triggering a page fault
    unsafe {
        asm!("mov {}, cr2", out(reg) address);
    }

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:x}", address);
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    panic!();
}

extern "x86-interrupt" fn ethernet_handler(_stack_frame: InterruptFrame) {
    crate::serial_println!("ethernet interrupt");

    unsafe {
        if let Some(dev) = &crate::driver::ETHERNET_DEVICE {
            let int_cause = dev.mmio_ind(0xC0);
            match int_cause {
                2 => crate::serial_println!("Packets transmetted"),
                _ => crate::serial_println!("interrupt cause: {}", int_cause),
            }
        }
    }
    PICS.obtain().end_all_interrupts();
}

extern "x86-interrupt" fn ata1_handler(_stack_frame: InterruptFrame) {
    PICS.obtain().end_all_interrupts();
}

extern "x86-interrupt" fn ata2_handler(_stack_frame: InterruptFrame) {
    PICS.obtain().end_all_interrupts();
}

#[derive(Default, Copy, Clone)]
#[repr(C, packed)]
struct InterruptGate {
    offset_low: u16,
    segment: u16,
    reserved: u8,
    /// type : 5;
    /// privilege_level : 2;
    /// segment_present : 1;
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

const SEG_PRESENT: u8 = 1 << 7;

#[repr(u8)]
enum GateType {
    TrapGate = 0b01111,
    IntGate = 0b01110,
}

impl InterruptGate {
    pub fn new(interrupt_handler: usize, gate_type: GateType) -> InterruptGate {
        let offset = interrupt_handler;

        InterruptGate {
            offset_low: offset as u16,
            segment: KERNEL_CODE_SEG.into(),
            reserved: 0,
            flags: gate_type as u8 | (KERNEL_CODE_SEG.get_privilege() << 5) | SEG_PRESENT,
            offset_mid: (offset >> 16) as u16,
            offset_high: (offset >> 32) as u32,
            zero: 0,
        }
    }
}

#[repr(C)]
struct Idt {
    entries: [InterruptGate; MAX_ENTRIES],
}

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    base: &'static Idt,
}

impl Idt {
    pub fn new() -> Idt {
        Idt {
            entries: [InterruptGate::default(); MAX_ENTRIES],
        }
    }

    pub fn insert(mut self, index: usize, interrupt_handler: usize, gate_type: GateType) -> Idt {
        let entry = InterruptGate::new(interrupt_handler, gate_type);
        self.entries[index] = entry;
        self
    }

    /// IDT must be valid
    pub unsafe fn load(self) {
        static mut IDT: MaybeUninit<Idt> = MaybeUninit::uninit();
        static mut IDTR: MaybeUninit<Idtr> = MaybeUninit::uninit();
        IDT = MaybeUninit::new(self);
        IDTR = MaybeUninit::new(Idtr::new(IDT.assume_init_ref()));

        asm!("lidt {}", sym IDTR);
    }
}

impl Idtr {
    pub fn new(idt: &'static Idt) -> Idtr {
        Idtr {
            limit: mem::size_of::<Idt>() as u16 - 1,
            base: idt,
        }
    }
}

pub enum InterruptIndex {
    Breakpoint = 1,
    DoubleFault = 8,
    PageFault = 14,
    Timer = PIC_1_OFFSET as isize,
    Keyboard,
    Ethernet = PIC_1_OFFSET as isize + 11,
    PrimaryATA = PIC_1_OFFSET as isize + 14,
    SecondaryATA,
}

impl From<InterruptIndex> for usize {
    fn from(index: InterruptIndex) -> Self {
        index as usize
    }
}

pub fn init() {
    use GateType::*;
    use InterruptIndex::*;
    let idt = Idt::new()
        .insert(Breakpoint.into(), breakpoint_handler as usize, TrapGate)
        .insert(DoubleFault.into(), double_fault_handler as usize, TrapGate)
        .insert(PageFault.into(), page_fault_handler as usize, TrapGate)
        .insert(Timer.into(), timer_handler as usize, IntGate)
        .insert(Keyboard.into(), keyboard_handler as usize, IntGate)
        .insert(Ethernet.into(), ethernet_handler as usize, IntGate)
        .insert(PrimaryATA.into(), ata1_handler as usize, IntGate)
        .insert(SecondaryATA.into(), ata2_handler as usize, IntGate);

    // SAFETY: IDT is filled with valid handlers of the correct type
    unsafe {
        idt.load();
    }
}

pub fn enable() {
    // SAFETY: This operation cannot fail
    unsafe {
        asm!("sti");
    }
}
