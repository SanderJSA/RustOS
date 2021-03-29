use core::sync::atomic::{AtomicBool, Ordering};

pub struct Spinlock(AtomicBool);

impl Spinlock {
    pub const fn new() -> Spinlock {
        Spinlock(AtomicBool::new(false))
    }

    #[allow(dead_code)]
    pub fn once(&self) -> bool {
        self.0
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    pub fn obtain(&self) {
        while self
            .0
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {}
    }

    pub fn release(&self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_case]
    fn call_once() {
        let mut result = false;
        let lock = Spinlock::new();

        if lock.once() {
            result = true;
        }
        if lock.once() {
            assert!(false);
        }

        assert!(result);
    }
}
