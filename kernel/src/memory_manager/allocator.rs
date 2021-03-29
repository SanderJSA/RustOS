use super::frame_allocator::FrameAllocator;
use super::PAGE_SIZE;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::null_mut;

pub struct Allocator {
    frame_allocator: UnsafeCell<FrameAllocator>,
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            frame_allocator: UnsafeCell::new(FrameAllocator::new()),
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == PAGE_SIZE && layout.align() == PAGE_SIZE {
            match (*self.frame_allocator.get()).allocate_frame() {
                None => null_mut(),
                Some(frame) => frame.base_addr as *mut u8,
            }
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_case]
    fn page_alloc() {
        let allocator = Allocator::new();
        let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();

        unsafe {
            let chunk = allocator.alloc(layout);
            //assert!(chunk.is_null());
            for i in 0..PAGE_SIZE as isize {
                *chunk.offset(i) = 0xab;
            }

            for i in 0..PAGE_SIZE as isize {
                //assert!(*chunk.offset(i) == 0xab);
            }
        }
    }
}
