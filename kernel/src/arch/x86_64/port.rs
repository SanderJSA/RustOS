#[cfg(target_arch = "x86_64")]
pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value);
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    asm!("in al, dx", in("dx") port, out("al") result);
    result
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn outw(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value);
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn inw(port: u16) -> u16 {
    let result: u16;
    asm!("in ax, dx", in("dx") port, out("ax") result);
    result
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn outd(port: u16, value: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") value);
}

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn ind(port: u16) -> u32 {
    let result: u32;
    asm!("in eax, dx", in("dx") port, out("eax") result);
    result
}
