/// Rustc may include calls to memset, memcopy and other functions which need to be implemented
/// when no system libc is provided

#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = c as u8;
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // We cannot use any iterators as they call memcpy in unoptimized builds
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, mut n: usize) -> *mut u8 {
    if src < dest && src as usize + n > dest as usize {
        while n > 0 {
            n -= 1;
            *dest.add(n) = *src.add(n);
        }
        dest
    } else {
        memcpy(dest, src, n)
    }
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let left = *s1.add(i);
        let right = *s2.add(i);
        if left != right {
            return left as i32 - right as i32;
        }
    }
    0
}
