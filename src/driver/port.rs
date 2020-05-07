#[cfg(target_arch = "x86_64")]
pub fn outb(port: u8, value: u8) {
    unsafe {
        llvm_asm!("outb %al, %dx" :: "{al}"(value), "{dx}"(port) :: "volatile");
    }
}

#[cfg(target_arch = "x86_64")]
pub fn inb(port: u8) -> u8 {
    unsafe {
        let result: u8;
        llvm_asm!("inb %dx, %al" : "={al}"(result) : "{dx}"(port) :: "volatile");
        result
    }
}
