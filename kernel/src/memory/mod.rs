pub mod frame;
pub mod frame_allocator;

pub const PAGE_SIZE: usize = 4096;

use self::frame_allocator::FrameAllocator;
use utils::lazy_static::LazyStatic;
#[allow(unused_imports)]
use x86_64::paging::tables::{self, EntryFlag};

static ALLOCATOR: LazyStatic<FrameAllocator> =
    LazyStatic::new(FrameAllocator::new);

/// Map a page of memory
/// An address is provided if none are given
/// This syscall is prone to data races
#[allow(dead_code)]
pub fn mmap(addr: Option<usize>, flags: u64) -> usize {
    let frame = ALLOCATOR.obtain().allocate_frame().expect("Out of memory");

    // Identity map if no address is given, use address otherwise
    let addr = addr.unwrap_or(frame.base_addr);

    tables::map_to(addr, frame.base_addr, flags, &mut ALLOCATOR.obtain());
    addr
}


use crate::test;

test!(basic_allocation {
    let page = mmap(None, EntryFlag::Writable as u64) as *mut u8;
    unsafe {
        *page.offset(0) = 204;
        *page.offset(1000) = 203;
        *page.offset(4095) = 204;

        assert!(*page.offset(0) == 204);
        assert!(*page.offset(1000) == 203);
        assert!(*page.offset(4095) == 204);
    }
});

test!(fixed_allocation {
    let page = mmap(Some(0xDEADBEEF000), EntryFlag::Writable as u64) as *mut u8;
    unsafe {
        *page.offset(0) = 204;
        *page.offset(423) = 203;
        *page.offset(4095) = 96;

        assert!(*page.offset(0) == 204);
        assert!(*page.offset(423) == 203);
        assert!(*page.offset(4095) == 96);
    }
});
