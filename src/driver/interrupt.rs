use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use super::cpuio::ChainedPics;
use crate::{print, println};
use driver::gdt;
use utils::lazy_static::Lazy;
use driver::port::inb;
use driver::PS2_keyboard;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static mut PICS: Lazy<ChainedPics> = Lazy::new();
pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init_pics() {
    unsafe { PICS.get(|| ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET))
        .initialize(); }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PICIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl PICIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_idt() {
    unsafe {
        IDT.breakpoint.set_handler_fn(breakpoint_handler);
        IDT.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        IDT[PICIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        IDT[PICIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        IDT.load();
    }
}

extern "x86-interrupt" fn
    double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT ERR:{}\n{:#?}", _error_code, stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe { PICS.get_already_init().notify_end_of_interrupt(PICIndex::Timer.as_u8()); }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let scan_code: u8 = inb(0x60);
    let symbol = PS2_keyboard::parse_scancode(scan_code);
    if let Some(symbol) = symbol {
        print!("{}", symbol);
    }
    unsafe { PICS.get_already_init().notify_end_of_interrupt(PICIndex::Keyboard.as_u8()); }
}
