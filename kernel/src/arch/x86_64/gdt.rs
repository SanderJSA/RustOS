use core::mem;

const KERNEL_RING: u8 = 0;
const USERLAND_RING: u8 = 3;
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const MAX_ENTRIES: usize = 64;
static mut GDT: GDT = GDT::new();

pub fn init() {
    unsafe {
        /* TODO: Add TSS
        static mut TSS: TaskStateSegment = TaskStateSegment::new();
                TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
                    const STACK_SIZE: usize = 4096;
                    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

                    let stack_start = VirtAddr::from_ptr(&STACK);
                    stack_start + STACK_SIZE
                };
        let tss_selector = GDT.add_entry(Descriptor::tss_segment(&TSS));
        load_tss(tss_selector);
        */

        let kernel_code_seg = GDT.add_entry(GDTEntry::new(0, 0xFFFFF, true, KERNEL_RING));
        let kernel_data_seg = GDT.add_entry(GDTEntry::new(0, 0xFFFFF, false, KERNEL_RING));

        let user_code_seg = GDT.add_entry(GDTEntry::new(0, 0xFFFFF, true, KERNEL_RING));
        let user_data_seg = GDT.add_entry(GDTEntry::new(0, 0xFFFFF, false, KERNEL_RING));

        GDT.load(kernel_data_seg, kernel_code_seg);
    }
}

const ACCESSED: u8 = 1;
const WRITEABLE: u8 = 1 << 1;
const EXPAND_DOWN: u8 = 1 << 2;
const EXECUTABLE: u8 = 1 << 3;
const DESC_TYPE: u8 = 1 << 4;
const PRIVILEGE_SHIFT: u8 = 5;
const SEG_PRESENT: u8 = 1 << 7;
const AVL: u8 = 1 << 4;
const CS_SIZE: u8 = 1 << 5;
const DEFAULT_OPERATION_SIZE: u8 = 1 << 6;
const GRANULARITY: u8 = 1 << 7;

#[derive(Default)]
#[repr(C)]
struct GDTEntry {
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
    base_addr_high: u16,
}

impl GDTEntry {
    pub const fn new(base_addr: u32, limit: u32, executable: bool, ring: u8) -> GDTEntry {
        let (executable, long_code_seg) = match executable {
            true => (EXECUTABLE, CS_SIZE),
            false => (0, 0),
        };
        GDTEntry {
            seg_lim_low: limit as u16,
            base_addr_low: base_addr as u16,
            base_addr_mid: (base_addr >> 16) as u8,
            access: WRITEABLE | DESC_TYPE | SEG_PRESENT | executable | (ring << PRIVILEGE_SHIFT),
            flags: ((limit >> 16) & 0xF) as u8 | AVL | CS_SIZE | GRANULARITY | long_code_seg,
            base_addr_high: (base_addr >> 24) as u16,
        }
    }
}

#[repr(C)]
struct GDT {
    limit: u16,
    base: [GDTEntry; MAX_ENTRIES],
    next_entry: usize,
}

impl GDT {
    pub const fn new() -> GDT {
        GDT {
            limit: mem::size_of::<[GDTEntry; MAX_ENTRIES]>() as u16,
            base: [GDTEntry::default(); MAX_ENTRIES],
            next_entry: 1, // 0th entry is the NULL Entry
        }
    }

    pub fn add_entry(&mut self, entry: GDTEntry) -> usize {
        let offset = self.next_entry;
        self.base[offset] = entry;
        self.next_entry += 1;
        offset
    }

    /// Loads the GDT and reloads segments
    /// GDT must be valid and given segments need to point to correct entries
    pub unsafe fn load(&'static self, data_seg: usize, code_seg: usize) {
        asm!("lgdt [{}]", in(reg) self);
        asm!("mov ds, ax",
             "mov ss, ax",
             "mov es, ax",
             "mov fs, ax",
             "mov gs, ax",
             in("ax") data_seg);
        asm!("jmp {}, 1:",
             "1:",
             in(reg) code_seg);
    }
}
