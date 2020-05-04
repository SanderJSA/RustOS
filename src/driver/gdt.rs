use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use crate::utils::lazy_static::*;
use x86_64::instructions::tables::load_tss;
use x86_64::instructions::segmentation::set_cs;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;


pub fn init() {
    static mut TSS: Lazy<TaskStateSegment> = Lazy::new();
    unsafe {
        TSS.init(|| {
            let mut tss = TaskStateSegment::new();
            tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
                const STACK_SIZE: usize = 4096;
                static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

                let stack_start = VirtAddr::from_ptr( &STACK);
                let stack_end = stack_start + STACK_SIZE;
                stack_end
            };
            tss
        });
    }

    static mut GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new();
    unsafe {
        GDT.get(||
        {
            let mut gdt = GlobalDescriptorTable::new();
            let code_selector =
                gdt.add_entry(Descriptor::kernel_code_segment());
            let tss_selector =
                gdt.add_entry(Descriptor::tss_segment(TSS.get_already_init()));
            (gdt, Selectors { code_selector, tss_selector })
        }).0.load();

        set_cs(GDT.get_already_init().1.code_selector);
        load_tss(GDT.get_already_init().1.tss_selector);
    }
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}