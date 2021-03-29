#[cfg(target_arch = "x86_64")]
pub unsafe fn outb(port: u16, value: u8) {
    llvm_asm!("outb %al, %dx" :: "{al}"(value), "{dx}"(port) :: "volatile");
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    llvm_asm!("inb %dx, %al" : "={al}"(result) : "{dx}"(port) :: "volatile");
    result
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn outw(port: u16, value: u16) {
    llvm_asm!("out %ax, %dx" :: "{ax}"(value), "{dx}"(port) :: "volatile");
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn inw(port: u16) -> u16 {
    let result: u16;
    llvm_asm!("in %dx, %ax" : "={ax}"(result) : "{dx}"(port) :: "volatile");
    result
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn outd(port: u16, value: u32) {
    llvm_asm!("out %eax, %dx" :: "{eax}"(value), "{dx}"(port) :: "volatile");
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn ind(port: u16) -> u32 {
    let result: u32;
    llvm_asm!("in %dx, %eax" : "={eax}"(result) : "{dx}"(port) :: "volatile");
    result
}
