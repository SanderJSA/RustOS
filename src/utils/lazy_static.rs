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

    pub fn init<F>(&mut self, builder: F)
        where F: Fn() -> T
    {
        if self.created.initialize() {
            self.value = MaybeUninit::new(builder());
        }
    }

    pub unsafe fn get_already_init(&'static mut self) -> &'static mut T {
        &mut *self.value.as_mut_ptr()
    }

    pub fn get<F>(&'static mut self, builder: F) -> &'static mut T
        where F: Fn() -> T
    {
        self.init(builder);
        unsafe { return self.get_already_init(); }
    }
}
