use crate::arch::paging::tables::EntryFlag;
use crate::memory_manager::{mmap, PAGE_SIZE};

pub struct Slab {
    block_size: usize,
    head: Option<&'static mut Block>,
}

struct Block {
    pub next: Option<&'static mut Block>,
}

impl Slab {
    /// Create a new slab allocator, `block_size` has to be at least 8 bytes wide
    pub fn new(block_size: usize) -> Slab {
        assert!(core::mem::size_of::<Block>() <= block_size);
        Slab {
            block_size,
            head: None,
        }
    }

    pub fn allocate(&mut self) -> *mut u8 {
        if self.head.is_none() {
            self.grow();
        }
        let block = self.head.take().unwrap();
        self.head = block.next.take();
        block as *mut _ as *mut u8
    }

    pub fn deallocate(&mut self, ptr: *mut u8) {
        let mut block = ptr as *mut Block;
        unsafe {
            // ptr has been allocated by allocate(),
            // It is a valid pointer large enough to fit a Block
            (*block).next = self.head.take();
            self.head = Some(&mut *(block));
        }

        // TODO check if current free block list allows us to free page
    }

    /// Allocate a page, fill it with `block_size` blocks and prepend them to the block list
    fn grow(&mut self) {
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
