use core::sync::atomic::{AtomicBool, Ordering};

pub struct Spinlock(AtomicBool);

impl Spinlock {
    pub const fn new() -> Spinlock {
        Spinlock(AtomicBool::new(false))
    }

    #[allow(dead_code)]
    pub fn once(&self) -> bool {
        !self.0.compare_and_swap(false, true, Ordering::SeqCst)
    }

    pub fn obtain(&self) {
        while self.0.compare_and_swap(false, true, Ordering::SeqCst) {}
    }

    pub fn release(&self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

use crate::test;

test!(call_once {
    let mut result = false;
    let lock = Spinlock::new();

    if lock.once() {
        result = true;
    }
    if lock.once() {
        assert!(false);
    }

    assert!(result);
});