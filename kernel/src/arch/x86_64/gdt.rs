use core::mem::{self, MaybeUninit};

const KERNEL_RING: u8 = 0;
const USERLAND_RING: u8 = 3;

pub const KERNEL_CODE_SEG: Segment = Segment::kernel(1);
const KERNEL_DATA_SEG: Segment = Segment::kernel(2);
const USER_CODE_SEG: Segment = Segment::userland(3);
const USER_DATA_SEG: Segment = Segment::userland(4);

const MAX_ENTRIES: usize = 5;

pub fn init() {
    let gdt = Gdt::new()
        .insert(KERNEL_CODE_SEG, GdtEntry::new(0, true, KERNEL_RING))
        .insert(KERNEL_DATA_SEG, GdtEntry::new(0, false, KERNEL_RING))
        .insert(USER_CODE_SEG, GdtEntry::new(0, true, USERLAND_RING))
        .insert(USER_DATA_SEG, GdtEntry::new(0, false, USERLAND_RING));

    // SAFETY: Gdt and Segments are valid
    unsafe {
        gdt.load(KERNEL_CODE_SEG, KERNEL_DATA_SEG);
    }
}

pub struct Segment(u16);

impl Segment {
    pub const fn new(offset: u16, privilege: u8) -> Segment {
        Segment((offset << 3) | (privilege as u16 & 0b11))
    }

    pub const fn kernel(offset: u16) -> Segment {
        Segment::new(offset, KERNEL_RING)
    }

    pub const fn userland(offset: u16) -> Segment {
        Segment::new(offset, USERLAND_RING)
    }

    pub const fn get_offset(&self) -> usize {
        (self.0 >> 3) as usize
    }

    pub const fn get_privilege(&self) -> u8 {
        (self.0 & 0b11) as u8
    }
}

impl From<Segment> for u16 {
    fn from(seg: Segment) -> Self {
        seg.0
    }
}

#[derive(Default, Debug)]
#[repr(C)]
struct GdtEntry {
    seg_lim_low: u16,
    base_addr_low: u16,
    base_addr_mid: u8,
    /// accessed : 1
    /// writeable : 1
    /// expand_down : 1
    /// executable : 1
    /// desc_type : 1
    /// privilege : 2
    /// seg_present : 1
    access: u8,
    /// seg_lim_high : 4
    /// avl : 1
    /// cs_size : 1
    /// default_operation_size : 1
    /// granularity : 1
    flags: u8,
    base_addr_high: u8,
}

const WRITEABLE: u8 = 1 << 1;
const EXECUTABLE: u8 = 1 << 3;
const DESC_TYPE: u8 = 1 << 4;
const PRIVILEGE_SHIFT: u8 = 5;
const SEG_PRESENT: u8 = 1 << 7;
const AVL: u8 = 1 << 4;
const CS_SIZE: u8 = 1 << 5;
const GRANULARITY: u8 = 1 << 7;

impl GdtEntry {
    pub const fn new(base_addr: u32, executable: bool, ring: u8) -> GdtEntry {
        let limit = 0xFFFFF;
        let (executable, long_code_seg) = match executable {
            true => (EXECUTABLE, CS_SIZE),
            false => (0, 0),
        };
        GdtEntry {
            seg_lim_low: limit as u16,
            base_addr_low: base_addr as u16,
            base_addr_mid: (base_addr >> 16) as u8,
            access: WRITEABLE | DESC_TYPE | SEG_PRESENT | executable | (ring << PRIVILEGE_SHIFT),
            flags: (limit >> 16) as u8 | AVL | GRANULARITY | long_code_seg,
            base_addr_high: (base_addr >> 24) as u8,
        }
    }

    pub fn is_code_segment(&self) -> bool {
        self.access & (SEG_PRESENT | EXECUTABLE) == (SEG_PRESENT | EXECUTABLE)
            && self.flags & CS_SIZE == CS_SIZE
    }

    pub fn is_data_segment(&self) -> bool {
        self.access & (SEG_PRESENT | EXECUTABLE) == SEG_PRESENT && self.flags & CS_SIZE == 0
    }
}

#[repr(C)]
struct Gdt {
    entries: [GdtEntry; MAX_ENTRIES],
}

#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: &'static Gdt,
}

impl Gdt {
    pub fn new() -> Gdt {
        Gdt {
            entries: Default::default(),
        }
    }

    pub fn insert(mut self, seg: Segment, entry: GdtEntry) -> Gdt {
        self.entries[seg.get_offset()] = entry;
        self
    }

    /// Loads the Gdt and reloads segments
    /// Is unsafe if GDT is not properly filled
    /// Panics if code_seg and data_seg don't point to their respective entries
    pub unsafe fn load(self, code_seg: Segment, data_seg: Segment) {
        assert!(self.entries[code_seg.get_offset()].is_code_segment());
        assert!(self.entries[data_seg.get_offset()].is_data_segment());

        static mut GDT: MaybeUninit<Gdt> = MaybeUninit::uninit();
        static mut GDTR: MaybeUninit<Gdtr> = MaybeUninit::uninit();
        GDT = MaybeUninit::new(self);
        GDTR = MaybeUninit::new(Gdtr::new(GDT.assume_init_ref()));

        asm!("lgdt {}", sym GDTR);
        reload_data_seg(data_seg);
        reload_code_seg(code_seg);
    }
}

impl Gdtr {
    pub fn new(gdt: &'static Gdt) -> Gdtr {
        Gdtr {
            limit: mem::size_of::<Gdt>() as u16,
            base: gdt,
        }
    }
}

unsafe fn reload_data_seg(seg: Segment) {
    // Set every data segment to segment selector
    asm!("mov ds, ax",
         "mov ss, ax",
         "mov es, ax",
         "mov fs, ax",
         "mov gs, ax",
         in("ax") u16::from(seg));
}

unsafe fn reload_code_seg(seg: Segment) {
    asm!("push {:r}",             // Push seg
         "lea {tmp}, [rip + 1f]", // Push rip
         "push {tmp}",            //
         "retfq",                 // Perform a far return which pops rip then cs
         "1:",                    //
         in(reg) u16::from(seg),
         tmp = out(reg) _);
}
