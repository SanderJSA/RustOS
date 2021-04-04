mod slab;
use crate::utils::lazy_static::LazyStatic;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use slab::Slab;

pub struct Allocator {
    slab_8: Slab,
    slab_16: Slab,
    slab_32: Slab,
    slab_64: Slab,
    slab_128: Slab,
    slab_512: Slab,
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            slab_8: Slab::new(8),
            slab_16: Slab::new(16),
            slab_32: Slab::new(32),
            slab_64: Slab::new(64),
            slab_128: Slab::new(128),
            slab_512: Slab::new(512),
        }
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        match (layout.size(), layout.align()) {
            (8, 8) => self.slab_8.allocate(),
            (16, 16) => self.slab_16.allocate(),
            (32, 32) => self.slab_32.allocate(),
            (64, 64) => self.slab_64.allocate(),
            (128, 128) => self.slab_128.allocate(),
            (512, 512) => self.slab_512.allocate(),
            _ => null_mut(),
        }
    }
}

pub struct LockedAllocator(LazyStatic<Allocator>);
unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.obtain().alloc(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory_manager::PAGE_SIZE;
    use core::alloc::Layout;

    #[test_case]
    fn page_alloc() {
        let mut allocator = Allocator::new();
        let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();

        unsafe {
            let chunk = allocator.alloc(layout);
            assert!(!chunk.is_null());
            for i in 0..PAGE_SIZE {
                *chunk.add(i) = 0xab;
            }

            for i in 0..PAGE_SIZE {
                assert!(*chunk.add(i) == 0xab);
            }
        }
    }
}
