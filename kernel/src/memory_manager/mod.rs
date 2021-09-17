pub mod allocator;
pub mod frame;
pub mod frame_allocator;

pub const PAGE_SIZE: usize = 4096;

use crate::arch::paging::tables;
use crate::utils::lazy_static::LazyStatic;
use frame_allocator::FrameAllocator;
use tables::EntryFlag;

static ALLOCATOR: LazyStatic<FrameAllocator> = LazyStatic::new(FrameAllocator::new);

/// Map a page of memory
/// An address is provided if none are given
/// This syscall is prone to data races
#[allow(dead_code)]
pub fn mmap(addr: Option<usize>, flags: u64) -> *mut u8 {
    let frame = ALLOCATOR.obtain().allocate_frame().expect("Out of memory");

    // Identity map if no address is given, use address otherwise
    let addr = addr.unwrap_or(frame.base_addr);

    tables::map_to(addr, frame.base_addr, flags, &mut ALLOCATOR.obtain());
    addr as *mut u8
}

pub fn munmap(_addr: *mut u8, _length: usize) {}

/// Direct maps virtual address to physical address
pub fn mmio_map(addr: usize, size: usize) {
    // TODO make pages unavailable to Frame Allocator
    // TODO fail if memory already allocated
    let page_align = |val| val / PAGE_SIZE * PAGE_SIZE;
    for i in page_align(addr)..page_align(addr + size + PAGE_SIZE - 1) {
        tables::map_to(
            i,
            i,
            EntryFlag::Writable as u64 + EntryFlag::WriteThrough as u64 + EntryFlag::NoCache as u64,
            &mut ALLOCATOR.obtain(),
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arch::paging::tables::EntryFlag;

    #[test_case]
    fn basic_allocation() {
        let page = mmap(None, EntryFlag::Writable as u64);
        unsafe {
            *page.offset(0) = 204;
            *page.offset(1000) = 203;
            *page.offset(4095) = 204;

            assert!(*page.offset(0) == 204);
            assert!(*page.offset(1000) == 203);
            assert!(*page.offset(4095) == 204);
        }
    }

    #[test_case]
    fn fixed_allocation() {
        let page = mmap(Some(0xBEEF0), EntryFlag::Writable as u64);
        unsafe {
            *page.offset(0) = 204;
            *page.offset(423) = 203;
            *page.offset(4095) = 96;

            assert!(*page.offset(0) == 204);
            assert!(*page.offset(423) == 203);
            assert!(*page.offset(4095) == 96);
        }
    }
}
