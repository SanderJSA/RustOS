use core::cmp::max;
use memory::{frame::Frame, PAGE_SIZE};
use x86_64::paging::memory_map;

pub const KERNEL_END: usize = 0x200000;

pub struct FrameAllocator {
    cur_frame: Frame,
    cur_region: memory_map::Region,
    region_index: u32,
}

impl FrameAllocator {
    /// Creates a FrameAllocator that allocates frames located after the end of the kernel
    pub fn new() -> FrameAllocator {
        let mut index = 0;
        while memory_map::get_region(index).end() <= KERNEL_END {
            index += 1;
        }

        let start_addr = max(
            memory_map::get_region(index).base_addr + PAGE_SIZE - 1,
            KERNEL_END,
        );

        FrameAllocator {
            cur_frame: Frame::from_address(start_addr),
            cur_region: memory_map::get_region(index),
            region_index: index,
        }
    }

    /// Returns a valid Page sized frame
    pub fn allocate_frame(&mut self) -> Option<Frame> {
        self.cur_frame.base_addr += PAGE_SIZE;
        if self.cur_frame.base_addr > self.cur_region.end() {
            if !self.next_region() {
                return None;
            }
            self.set_start_frame();
        }

        Some(self.cur_frame)
    }

    /// sets the first frame to the start of the current region
    fn set_start_frame(&mut self) {
        self.cur_frame =
            Frame::from_address(self.cur_region.base_addr + PAGE_SIZE - 1);
    }

    /// Go to next region, returns false if not possible
    fn next_region(&mut self) -> bool {
        self.region_index += 1;
        if self.region_index == memory_map::region_count() {
            return false;
        }

        self.cur_region = memory_map::get_region(self.region_index);
        true
    }
}
