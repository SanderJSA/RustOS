use super::spinlock::Spinlock;
use core::cell::UnsafeCell;
use core::mem::{replace, MaybeUninit};
use core::ops::{Deref, DerefMut};

pub struct LazyStatic<T, F = fn() -> T> {
    lock: Spinlock,
    data: UnsafeCell<MaybeUninit<T>>,
    builder: UnsafeCell<Option<F>>,
}

pub struct LazyGuard<'a, T> {
    data: &'a mut T,
    lock: &'a Spinlock,
}

impl<T, F> LazyStatic<T, F> {
    pub const fn new(builder: F) -> Self {
        LazyStatic {
            lock: Spinlock::new(),
            data: UnsafeCell::new(MaybeUninit::uninit()),
            builder: UnsafeCell::new(Some(builder)),
        }
    }
}

impl<T, F: FnOnce() -> T> LazyStatic<T, F> {
    pub fn obtain(&self) -> LazyGuard<T> {
        self.lock.obtain();
        if let Some(f) = replace(unsafe { &mut *self.builder.get() }, None) {
            unsafe {
                (*self.data.get()).as_mut_ptr().write((f)());
            }
        }

        LazyGuard {
            data: unsafe { &mut *(*self.data.get()).as_mut_ptr() },
            lock: &self.lock,
        }
    }
}

impl<'a, T> Deref for LazyGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<'a, T> DerefMut for LazyGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}

impl<'a, T> Drop for LazyGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.release();
    }
}

unsafe impl<T, F> Sync for LazyStatic<T, F> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test_case]
    fn lock_unlock() {
        let lazy: LazyStatic<u32> = LazyStatic::new(|| 10);
        assert!(lazy.obtain().eq(&10));
        assert!(lazy.obtain().eq(&10));
    }
}
