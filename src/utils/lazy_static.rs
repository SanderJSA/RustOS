use core::mem::MaybeUninit;
use super::spinlock;

pub struct Lazy<T> {
    value: MaybeUninit<T>,
    created: spinlock::Once,
}

impl <T> Lazy<T> {
    pub const fn new() -> Lazy<T> {
        Lazy { value: MaybeUninit::uninit(), created: spinlock::Once::new() }
    }

    pub fn get<F>(&'static mut self, builder: F) -> &'static mut T
        where F: Fn() -> T
    {
        if self.created.initialize() {
            self.value = MaybeUninit::new(builder());
        }
        unsafe { &mut *self.value.as_mut_ptr() }
    }
}
