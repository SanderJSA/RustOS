use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Release, Acquire, Relaxed};

static mut LOCKED : AtomicBool = AtomicBool::new(false);

pub fn obtain_lock() {
    unsafe {
        while LOCKED.compare_and_swap(false, true, Acquire) {}
    }
}

pub fn release_lock() {
    unsafe {
        LOCKED.store(false, Release);
    }
}

// Implements a poor man's version of a call_once
pub struct Once(AtomicBool);

impl Once {
    pub const fn new() -> Once {
        Once(AtomicBool::new(false))
    }

    pub fn initialize(&mut self) -> bool {
        !self.0.compare_and_swap(false, true, Relaxed)
    }
}