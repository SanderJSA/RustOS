use super::pic::ChainedPics;
use super::{gdt, port};
use crate::driver::ps2_keyboard;
use crate::println;
use crate::utils::lazy_static::LazyStatic;
use x86_64_crate::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};

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

    fn as_usize(self) -> usize {
        (self.as_u8()) as usize
    }
}

pub unsafe fn init_idt() {
    IDT.breakpoint.set_handler_fn(breakpoint_handler);
    IDT.page_fault.set_handler_fn(page_fault_handler);
    IDT.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    IDT[PICIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
    IDT[PICIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
    IDT[PICIndex::PrimaryATA.as_usize()].set_handler_fn(ata1_handler);
    IDT[PICIndex::SecondaryATA.as_usize()].set_handler_fn(ata2_handler);
    IDT.load();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT ERR:{}\n{:#?}",
        _error_code, stack_frame
    );
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    PICS.obtain()
        .notify_end_of_interrupt(PICIndex::Timer.as_u8());
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let scan_code: u8 = unsafe {
        // KEYBOARD_PORT exists
        port::inb(KEYBOARD_PORT)
    };
    ps2_keyboard::update_stdin(scan_code);
    PICS.obtain()
        .notify_end_of_interrupt(PICIndex::Keyboard.as_u8());
}
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let address: usize;
    unsafe {
        // Will always return address of invalid_access
        llvm_asm!("mov %cr2, %rax" : "={rax}"(address) ::: "volatile");
    }

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {}", address);
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    panic!();
}

extern "x86-interrupt" fn ata1_handler(_stack_frame: &mut InterruptStackFrame) {
    PICS.obtain().end_all_interrupts();
}

extern "x86-interrupt" fn ata2_handler(_stack_frame: &mut InterruptStackFrame) {
    PICS.obtain().end_all_interrupts();
}
