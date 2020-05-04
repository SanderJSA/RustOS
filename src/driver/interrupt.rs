use super::cpuio::ChainedPics;
use crate::utils::lazy_static::*;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

//pub static PICS: Lazy<ChainedPics, fnOnce() -> ChainedPics> =
//    Lazy::new(|| ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET));


pub fn init_idt() {
    pub static mut IDT: Lazy<InterruptDescriptorTable> = Lazy::new();

    unsafe {
        IDT.get(InterruptDescriptorTable::new).breakpoint.set_handler_fn(breakpoint_handler);
        IDT.get_already_init().double_fault.set_handler_fn(double_fault_handler);
        IDT.get_already_init().load();
    }
}

extern "x86-interrupt" fn double_fault_handler(
stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}