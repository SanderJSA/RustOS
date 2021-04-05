mod slab;
use crate::arch::paging::tables::EntryFlag;
use crate::memory_manager::{mmap, munmap, PAGE_SIZE};
use crate::utils::lazy_static::LazyStatic;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use slab::Slab;

#[global_allocator]
static GLOBAL_ALLOCATOR: LockedAllocator = LockedAllocator {
    0: LazyStatic::new(Allocator::new),
};

#[alloc_error_handler]
fn on_oom(layout: Layout) -> ! {
    panic!(
        "Allocator ran out of memory while attempting to allocate {:?}",
        layout
    );
}

pub struct Allocator {
    slab_8: Slab,
    slab_16: Slab,
    slab_32: Slab,
    slab_64: Slab,
    slab_128: Slab,
    slab_256: Slab,
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
            slab_256: Slab::new(256),
            slab_512: Slab::new(512),
        }
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if layout.size() <= 8 && layout.align() <= 8 {
            self.slab_8.allocate()
        } else if layout.size() <= 16 && layout.align() <= 16 {
            self.slab_16.allocate()
        } else if layout.size() <= 32 && layout.align() <= 32 {
            self.slab_32.allocate()
        } else if layout.size() <= 64 && layout.align() <= 64 {
            self.slab_64.allocate()
        } else if layout.size() <= 128 && layout.align() <= 128 {
            self.slab_128.allocate()
        } else if layout.size() <= 256 && layout.align() <= 256 {
            self.slab_256.allocate()
        } else if layout.size() <= 512 && layout.align() <= 512 {
            self.slab_512.allocate()
        } else if layout.size() <= PAGE_SIZE && layout.align() <= PAGE_SIZE {
            mmap(None, EntryFlag::Writable as u64)
        } else {
            null_mut()
        }
    }

    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if layout.size() <= 8 && layout.align() <= 8 {
            self.slab_8.deallocate(ptr);
        } else if layout.size() <= 16 && layout.align() <= 16 {
            self.slab_16.deallocate(ptr);
        } else if layout.size() <= 32 && layout.align() <= 32 {
            self.slab_32.deallocate(ptr);
        } else if layout.size() <= 64 && layout.align() <= 64 {
            self.slab_64.deallocate(ptr);
        } else if layout.size() <= 128 && layout.align() <= 128 {
            self.slab_128.deallocate(ptr);
        } else if layout.size() <= 256 && layout.align() <= 256 {
            self.slab_256.deallocate(ptr);
        } else if layout.size() <= 512 && layout.align() <= 512 {
            self.slab_512.deallocate(ptr);
        } else {
            munmap(ptr, PAGE_SIZE);
        }
    }
}

pub struct LockedAllocator(LazyStatic<Allocator>);
unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.obtain().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.obtain().dealloc(ptr, layout)
    }
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

        let chunk = allocator.alloc(layout);
        unsafe {
            assert!(!chunk.is_null());
            for i in 0..PAGE_SIZE {
                *chunk.add(i) = 0xab;
            }

            for i in 0..PAGE_SIZE {
                assert!(*chunk.add(i) == 0xab);
            }
        }
        allocator.dealloc(chunk, layout);
    }

    #[test_case]
    fn allocate_bytes() {
        let mut allocator = Allocator::new();
        let layout = Layout::from_size_align(8, 8).unwrap();

        let ptr1 = allocator.alloc(layout) as *mut i64;
        let ptr2 = allocator.alloc(layout) as *mut i64;
        let ptr3 = allocator.alloc(layout) as *mut i64;
        unsafe {
            *ptr1 = -5810;
            *ptr2 = -1;
            *ptr3 = 9999900009;

            assert!(*ptr1 == -5810);
            assert!(*ptr2 == -1);
            assert!(*ptr3 == 9999900009);
        }

        allocator.dealloc(ptr1 as *mut u8, layout);
        allocator.dealloc(ptr2 as *mut u8, layout);
        allocator.dealloc(ptr3 as *mut u8, layout);
    }

    #[test_case]
    fn two_pages_allocations() {
        let mut allocator = Allocator::new();
        let layout = Layout::from_size_align(512, 512).unwrap();

        let ptr1 = allocator.alloc(layout) as *mut i64;
        let ptr2 = allocator.alloc(layout) as *mut i64;
        let ptr3 = allocator.alloc(layout) as *mut i64;
        unsafe {
            *ptr1 = -5810;
            *ptr2 = -1;
            *ptr3 = 9999900009;

            assert!(*ptr1 == -5810);
            assert!(*ptr2 == -1);
            assert!(*ptr3 == 9999900009);
        }

        allocator.dealloc(ptr1 as *mut u8, layout);
        allocator.dealloc(ptr2 as *mut u8, layout);
        allocator.dealloc(ptr3 as *mut u8, layout);
    }
}
