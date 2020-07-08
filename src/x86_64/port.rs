#[cfg(target_arch = "x86_64")]
pub fn outb(port: u16, value: u8) {
    unsafe {
        llvm_asm!("outb %al, %dx" :: "{al}"(value), "{dx}"(port) :: "volatile");
    }
}

#[cfg(target_arch = "x86_64")]
pub fn inb(port: u16) -> u8 {
    unsafe {
        let result: u8;
        llvm_asm!("inb %dx, %al" : "={al}"(result) : "{dx}"(port) :: "volatile");
        result
    }
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub fn outw(port: u16, value: u16) {
    unsafe {
        llvm_asm!("out %ax, %dx" :: "{ax}"(value), "{dx}"(port) :: "volatile");
    }
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub fn inw(port: u16) -> u16 {
    unsafe {
        let result: u16;
        llvm_asm!("in %dx, %ax" : "={ax}"(result) : "{dx}"(port) :: "volatile");
        result
    }
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub fn outd(port: u16, value: u32) {
    unsafe {
        llvm_asm!("out %eax, %dx" :: "{eax}"(value), "{dx}"(port) :: "volatile");
    }
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub fn ind(port: u16) -> u32 {
    unsafe {
        let result: u32;
        llvm_asm!("in %dx, %eax" : "={eax}"(result) : "{dx}"(port) :: "volatile");
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum ExitCode {
    Success = 0x10,
    Failure = 0x11,
}

#[allow(dead_code)]
pub fn exit(exit_code: ExitCode) {
    outb(0xf4, exit_code as u8);
}
