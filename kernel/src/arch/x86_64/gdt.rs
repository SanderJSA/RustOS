use core::mem::{self, MaybeUninit};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

const KERNEL_RING: u8 = 0;
const USERLAND_RING: u8 = 3;

const KERNEL_CODE_SEG: Segment = Segment::new(1, KERNEL_RING);
const KERNEL_DATA_SEG: Segment = Segment::new(2, KERNEL_RING);
const USER_CODE_SEG: Segment = Segment::new(3, USERLAND_RING);
const USER_DATA_SEG: Segment = Segment::new(4, USERLAND_RING);

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

struct Segment(u64);

impl Segment {
    pub const fn new(offset: u64, privilege: u8) -> Segment {
        Segment((offset << 3) | (privilege as u64 & 0b11))
    }

    pub const fn get_offset(&self) -> usize {
        (self.0 >> 3) as usize
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
            flags: (limit >> 16) as u8 | AVL | CS_SIZE | GRANULARITY | long_code_seg,
            base_addr_high: (base_addr >> 24) as u8,
        }
    }
}

#[repr(C)]
struct Gdt {
    entries: [GdtEntry; MAX_ENTRIES],
}

#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: *const Gdt,
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
    /// Gdt must be valid
    pub unsafe fn load(self, code_seg: Segment, data_seg: Segment) {
        static mut GDT: MaybeUninit<(Gdtr, Gdt)> = MaybeUninit::uninit();
        GDT = MaybeUninit::new((Gdtr::new(&self), self));

        asm!("lgdt {}", sym GDT);
        asm!("mov ds, ax",
             "mov ss, ax",
             "mov es, ax",
             "mov fs, ax",
             "mov gs, ax",
             in("ax") *((&data_seg as *const _) as *const u64));
        asm!("push {}",
             "lea  rax, [rip + 1f]",
             "push rax",
             "retfq",
             "1:",
             in(reg) *((&code_seg as *const _) as *const u64),
             out("rax") _);
    }
}

impl Gdtr {
    pub fn new(gdt: &Gdt) -> Gdtr {
        Gdtr {
            limit: mem::size_of::<[GdtEntry; MAX_ENTRIES]>() as u16 - 1,
            base: gdt,
        }
    }
}
