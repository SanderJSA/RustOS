use core::arch::asm;
/// Send a u8 to IO `port`
///
/// # Safety
///
/// Side-effects caused by function must be expected by caller
/// 'port' must expect to receive `value`
pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value);
}

/// Read a u8 from IO `port`
///
/// # Safety
///
/// Side-effects caused by function must be expected by caller
pub unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    asm!("in al, dx", in("dx") port, out("al") result);
    result
}

/// Send a u16 to IO `port`
///
/// # Safety
///
/// Side-effects caused by function must be expected by caller
/// 'port' must expect to receive `value`
pub unsafe fn outw(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value);
}

/// Read a u16 from IO `port`
///
/// # Safety
///
/// Side-effects caused by function must be expected by caller
pub unsafe fn inw(port: u16) -> u16 {
    let result: u16;
    asm!("in ax, dx", in("dx") port, out("ax") result);
    result
}

/// Send a u32 to IO `port`
///
/// # Safety
///
/// Side-effects caused by function must be expected by caller
/// 'port' must expect to receive `value`
pub unsafe fn outd(port: u16, value: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") value);
}

/// Read a u32 from IO `port`
///
/// # Safety
///
/// Side-effects caused by function must be expected by caller
pub unsafe fn ind(port: u16) -> u32 {
    let result: u32;
    asm!("in eax, dx", in("dx") port, out("eax") result);
    result
}
