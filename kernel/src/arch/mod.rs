mod x86_64;
pub use x86_64::*;

pub fn init() {
    gdt::init();
    interrupt::init();
    pic::init();
    interrupt::enable();
    pci::init();
}
