use x86_64_crate::instructions::segmentation::set_cs;
use x86_64_crate::instructions::tables::load_tss;
use x86_64_crate::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64_crate::structures::tss::TaskStateSegment;
use x86_64_crate::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

pub fn init() {
    unsafe {
        TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        let code_selector = GDT.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = GDT.add_entry(Descriptor::tss_segment(&TSS));

        GDT.load();
        set_cs(code_selector);
        load_tss(tss_selector);
    }
}
