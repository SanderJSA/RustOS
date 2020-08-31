use memory::PAGE_SIZE;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Frame {
    pub base_addr: usize,
}

impl Frame {
    /// Create frame that points to the start of a page
    pub fn from_address(address: usize) -> Frame {
        Frame {
            base_addr: (address / PAGE_SIZE) * PAGE_SIZE
        }
    }
}
