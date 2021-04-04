use crate::arch::paging::tables::EntryFlag;
use crate::memory_manager::{mmap, PAGE_SIZE};

pub struct Slab {
    block_size: usize,
    len: usize,
    head: Option<&'static mut Block>,
}

struct Block {
    next: Option<&'static mut Block>,
}

impl Slab {
    pub fn new(block_size: usize) -> Slab {
        Slab {
            block_size,
            len: 0,
            head: None,
        }
    }

    pub fn allocate(&mut self) -> *mut u8 {
        core::ptr::null_mut()
    }

    /// Allocate a page, fill it with `block_size` blocks and prepend them to the block list
    pub fn grow(&mut self) {
        let page = mmap(None, EntryFlag::Writable as u64);

        // Fill as many Blocks as an mmap page can fit
        for i in (0..PAGE_SIZE).step_by(self.block_size) {
            unsafe {
                // All memory access in [page, page + PAGE_SIZE[ is valid
                let mut block = page.add(i) as *mut Block;
                (*block).next = match i + self.block_size {
                    x if x >= PAGE_SIZE => self.head.take(),
                    x => Some(&mut *(page.add(x) as *mut Block)),
                };
            }
        }
        // Set slab head to first block of this page
        self.head = unsafe {
            // The Block at address `page` is allocated and initialized
            Some(&mut *(page as *mut Block))
        };
    }
}
