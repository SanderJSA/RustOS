pub mod frame;
pub mod frame_allocator;

pub const PAGE_SIZE: usize = 4096;

use x86_64::paging::tables;
use self::frame_allocator::FrameAllocator;
use utils::lazy_static::Lazy;

static mut ALLOCATOR: Lazy<FrameAllocator> = Lazy::new();

/// Map a page of memory
/// An address is provided if none are given
/// This syscall is prone to data races
pub fn mmap(addr: Option<usize>, flags: u64) -> usize {

    unsafe {
        let frame =  ALLOCATOR
            .get(FrameAllocator::new)
            .allocate_frame()
            .expect("Out of memory");

        // Identity map if no address is given, use address otherwise
        let addr = addr.unwrap_or(frame.base_addr);

        tables::map_to(addr, frame.base_addr, flags,
                       &mut ALLOCATOR.get_already_init());
        addr
    }
}