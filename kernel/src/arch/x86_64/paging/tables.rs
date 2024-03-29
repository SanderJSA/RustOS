use crate::memory_manager::frame::Frame;
use crate::memory_manager::frame_allocator::FrameAllocator;
use crate::memory_manager::PAGE_SIZE;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

const ENTRY_COUNT: usize = 512;

#[allow(dead_code)]
#[repr(u64)]
pub enum EntryFlag {
    Present = 1,
    Writable = 1 << 1,
    UserAccessible = 1 << 2,
    WriteThrough = 1 << 3,
    NoCache = 1 << 4,
    Accessed = 1 << 5,
    Dirty = 1 << 6,
    HugePage = 1 << 7,
    Global = 1 << 8,
    NoExecute = 1 << 63,
}

#[repr(transparent)]
pub struct Entry(u64);

impl Entry {
    /// Frees entry
    #[allow(dead_code)]
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Returns true if entry contains given flags
    pub fn contains(&self, flags: u64) -> bool {
        self.0 & flags != 0
    }

    /// Returns Frame containing address of entry
    pub fn address(&self) -> Option<Frame> {
        if self.contains(EntryFlag::Present as u64) {
            Some(Frame::from_address(self.0 as usize & 0x000F_FFFF_FFFF_F000))
        } else {
            None
        }
    }

    /// Sets Entry to given values
    pub fn set(&mut self, frame: Frame, flags: u64) {
        self.0 = frame.base_addr as u64 | flags;
    }
}

/// Table Level
pub trait TableLevel {
    fn index(page: usize) -> usize;
}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {
    fn index(page: usize) -> usize {
        (page >> 27) & 0o777
    }
}
impl TableLevel for Level3 {
    fn index(page: usize) -> usize {
        (page >> 18) & 0o777
    }
}
impl TableLevel for Level2 {
    fn index(page: usize) -> usize {
        (page >> 9) & 0o777
    }
}
impl TableLevel for Level1 {
    fn index(page: usize) -> usize {
        page & 0o777
    }
}

/// Tables that point to other tables
pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

#[repr(transparent)]
pub struct Table<L: TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

/// Methods that apply to every tables
impl<L> Table<L>
where
    L: TableLevel,
{
    /// Empties table
    #[allow(dead_code)]
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

// Returns the level 4 table by doing 4 recursion on level 4 table
pub fn get_level4() -> &'static mut Table<Level4> {
    unsafe { &mut *(0xFFFF_FFFF_FFFF_F000 as *mut Table<Level4>) }
}

/// Methods that only apply to tables that point to other tables
impl<L> Table<L>
where
    L: HierarchicalLevel,
{
    /// Get the next table address
    fn next_table_addr(&self, index: usize) -> Option<usize> {
        if self[index].contains(EntryFlag::Present as u64) {
            let table_addr = (self as *const _) as usize;
            Some((table_addr << 9) | (index << 12))
        } else {
            None
        }
    }

    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_addr(index)
            .map(|address| unsafe { &*(address as *const Table<L::NextLevel>) })
    }

    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_addr(index)
            .map(|address| unsafe { &mut *(address as *mut Table<L::NextLevel>) })
    }

    fn create_table(
        &mut self,
        index: usize,
        allocator: &mut FrameAllocator,
    ) -> &mut Table<L::NextLevel> {
        if self.next_table(index).is_none() {
            let frame = allocator
                .allocate_frame()
                .expect("Could not allocate any frame");
            self[index].set(
                frame,
                (EntryFlag::Present as u64) | (EntryFlag::Writable as u64),
            );
        }

        self.next_table_mut(index).unwrap()
    }
}

impl<L> Index<usize> for Table<L>
where
    L: TableLevel,
{
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L>
where
    L: TableLevel,
{
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

#[allow(dead_code)]
pub fn translate_addr(virt_addr: usize) -> Option<usize> {
    let offset = virt_addr % PAGE_SIZE;
    let page = virt_addr / PAGE_SIZE;

    let p4 = get_level4();
    p4.next_table(Level4::index(page))
        .and_then(|p3| p3.next_table(Level3::index(page)))
        .and_then(|p2| p2.next_table(Level2::index(page)))
        .and_then(|p1| p1[Level1::index(page)].address())
        .map(|frame| frame.base_addr + offset)
}

pub fn map_to(virt_addr: usize, phys_addr: usize, flags: u64, allocator: &mut FrameAllocator) {
    let page = virt_addr / PAGE_SIZE;

    get_level4()
        .create_table(Level4::index(page), allocator)
        .create_table(Level3::index(page), allocator)
        .create_table(Level2::index(page), allocator)
        .index_mut(Level1::index(page))
        .set(
            Frame::from_address(phys_addr),
            flags | EntryFlag::Present as u64,
        );
}
