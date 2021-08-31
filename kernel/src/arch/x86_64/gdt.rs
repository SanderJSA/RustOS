use core::mem;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

const KERNEL_RING: u8 = 0;
#[allow(unused)]
const USERLAND_RING: u8 = 3;

const KERNEL_CODE_OFF: usize = 1;
const KERNEL_DATA_OFF: usize = 2;
const USER_CODE_OFF: usize = 3;
const USER_DATA_OFF: usize = 4;

const MAX_ENTRIES: usize = 8;

pub fn init() {
    /* TODO: Add TSS
    static mut TSS: TaskStateSegment = TaskStateSegment::new();
            TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
                const STACK_SIZE: usize = 4096;
                static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

                let stack_start = VirtAddr::from_ptr(&STACK);
                stack_start + STACK_SIZE
            };
    let tss_selector = Gdt.add_entry(Descriptor::tss_segment(&TSS));
    load_tss(tss_selector);
    */

    let gdt = Gdt::new()
        .insert(KERNEL_CODE_OFF, GdtEntry::new(0, true, KERNEL_RING))
        .insert(KERNEL_DATA_OFF, GdtEntry::new(0, false, KERNEL_RING))
        .insert(USER_CODE_OFF, GdtEntry::new(0, true, USERLAND_RING))
        .insert(USER_DATA_OFF, GdtEntry::new(0, false, USERLAND_RING));

    unsafe {
        gdt.load();
    }
}

#[allow(unused)]
const ACCESSED: u8 = 1;
const WRITEABLE: u8 = 1 << 1;
#[allow(unused)]
const EXPAND_DOWN: u8 = 1 << 2;
const EXECUTABLE: u8 = 1 << 3;
const DESC_TYPE: u8 = 1 << 4;
const PRIVILEGE_SHIFT: u8 = 5;
const SEG_PRESENT: u8 = 1 << 7;
const AVL: u8 = 1 << 4;
const CS_SIZE: u8 = 1 << 5;
#[allow(unused)]
const DEFAULT_OPERATION_SIZE: u8 = 1 << 6;
const GRANULARITY: u8 = 1 << 7;

#[derive(Default)]
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
            flags: ((limit >> 16) & 0xF) as u8 | AVL | CS_SIZE | GRANULARITY | long_code_seg,
            base_addr_high: (base_addr >> 24) as u8,
        }
    }
}

#[repr(C)]
struct Gdt {
    limit: u16,
    base: [GdtEntry; MAX_ENTRIES],
}

impl Gdt {
    pub const fn new() -> Gdt {
        let entries;
        // SAFETY: A NULL entry is a valid entry and GdtEntry is 64 bits wide
        unsafe {
            entries =
                mem::transmute::<[u64; MAX_ENTRIES], [GdtEntry; MAX_ENTRIES]>([0u64; MAX_ENTRIES]);
        };

        Gdt {
            limit: mem::size_of::<[GdtEntry; MAX_ENTRIES]>() as u16,
            base: entries,
        }
    }

    pub fn insert(mut self, index: usize, entry: GdtEntry) -> Gdt {
        self.base[index] = entry;
        self
    }

    /// Loads the Gdt and reloads segments
    /// Gdt must be valid
    pub unsafe fn load(self) {
        static mut GDT: Option<Gdt> = None;
        GDT = Some(self);

        asm!("lgdt [{}]", in(reg) GDT.as_ref().unwrap());
        asm!("mov ds, ax",
             "mov ss, ax",
             "mov es, ax",
             "mov fs, ax",
             "mov gs, ax",
             in("ax") KERNEL_DATA_OFF);
        asm!("ljmp {}, 1f", "1:", const KERNEL_CODE_OFF);
    }
}
