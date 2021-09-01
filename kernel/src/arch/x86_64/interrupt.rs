use super::gdt::KERNEL_CODE_SEG;
use super::pic::ChainedPics;
use super::{gdt, port};
use crate::driver::ps2_keyboard;
use crate::println;
use crate::utils::lazy_static::LazyStatic;
use core::mem::{self, MaybeUninit};
use x86_64_crate::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};

const MAX_ENTRIES: usize = 256;

const KEYBOARD_PORT: u16 = 0x60;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
pub static PICS: LazyStatic<ChainedPics> =
    LazyStatic::new(|| ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET));

pub fn init_pics() {
    unsafe {
        // Both PIC are created with valid offsets
        PICS.obtain().initialize();
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PICIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    PrimaryATA = PIC_1_OFFSET + 14,
    SecondaryATA,
}

impl PICIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT ERR:{}\n{:#?}",
        _error_code, stack_frame
    );
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    PICS.obtain()
        .notify_end_of_interrupt(PICIndex::Timer.as_u8());
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    let scan_code: u8 = unsafe {
        // KEYBOARD_PORT exists
        port::inb(KEYBOARD_PORT)
    };
    ps2_keyboard::update_stdin(scan_code);
    PICS.obtain()
        .notify_end_of_interrupt(PICIndex::Keyboard.as_u8());
}
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let address: usize;
    unsafe {
        // Will always return address of invalid_access
        asm!("", out("rax") address);
    }

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {}", address);
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    panic!();
}

extern "x86-interrupt" fn ata1_handler(_stack_frame: InterruptStackFrame) {
    PICS.obtain().end_all_interrupts();
}

extern "x86-interrupt" fn ata2_handler(_stack_frame: InterruptStackFrame) {
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
        asm!("xchg bx, bx");
        static mut IDT: MaybeUninit<Idt> = MaybeUninit::uninit();
        static mut IDTR: MaybeUninit<Idtr> = MaybeUninit::uninit();
        IDT = MaybeUninit::new(self);
        IDTR = MaybeUninit::new(Idtr::new(IDT.assume_init_ref()));

        asm!("lidt {}", sym IDTR);
        asm!("xchg bx, bx");
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
        .insert(PrimaryATA.into(), ata1_handler as usize, IntGate)
        .insert(SecondaryATA.into(), ata2_handler as usize, IntGate);

    // SAFETY: IDT is filled with valid handlers of the correct type
    unsafe {
        asm!("xchg bx, bx");
        idt.load();
    }
}

pub fn enable() {
    // SAFETY: This operation cannot fail
    unsafe {
        asm!("sti");
    }
}
