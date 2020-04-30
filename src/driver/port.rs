#[cfg(target_arch = "x86_64")]
pub fn out(port: u16, value: u32) {
    unsafe {
        asm!("outl %eax, (%dx)" :: "{eax}"(value), "{dx}"(port));
    }
}