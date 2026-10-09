#![no_std]

pub const GDT_ENTRIES: usize = 7;
pub const GDT_KERNEL_CODE: u16 = 0x08;
pub const GDT_KERNEL_DATA: u16 = 0x10;
pub const GDT_USER_DATA: u16 = 0x18;
pub const GDT_USER_CODE: u16 = 0x20;
pub const GDT_TSS: u16 = 0x28;

#[repr(C, packed)]
pub struct GdtEntry {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_mid: u8,
    pub access: u8,
    pub granularity: u8,
    pub base_high: u8,
}

#[repr(C, packed)]
pub struct GdtPtr {
    pub limit: u16,
    pub base: u64,
}

#[repr(C, align(16))]
pub struct Tss {
    pub reserved0: u32,
    pub rsp: [u64; 3],
    pub reserved1: u64,
    pub ist: [u64; 7],
    pub reserved2: u64,
    pub reserved3: u16,
    pub iomap_base: u16,
}

static mut GDT: [GdtEntry; GDT_ENTRIES] = [GdtEntry {
    limit_low: 0,
    base_low: 0,
    base_mid: 0,
    access: 0,
    granularity: 0,
    base_high: 0,
}; GDT_ENTRIES];

static mut GDT_PTR: GdtPtr = GdtPtr { limit: 0, base: 0 };
static mut TSS: Tss = Tss {
    reserved0: 0,
    rsp: [0; 3],
    reserved1: 0,
    ist: [0; 7],
    reserved2: 0,
    reserved3: 0,
    iomap_base: 0,
};

pub unsafe fn gdt_set_entry(index: usize, base: u64, limit: u64, access: u8, granularity: u8) {
    if index >= GDT_ENTRIES { return; }
    let entry = &mut GDT[index];
    entry.limit_low = limit as u16;
    entry.base_low = base as u16;
    entry.base_mid = (base >> 16) as u8;
    entry.access = access;
    entry.granularity = ((limit >> 16) & 0x0F) as u8 | (granularity & 0xF0);
    entry.base_high = (base >> 24) as u8;
}

pub unsafe fn gdt_init() {
    GDT_PTR.limit = core::mem::size_of::<GdtEntry>() * GDT_ENTRIES as usize - 1;
    GDT_PTR.base = &GDT as *const _ as u64;
    gdt_set_entry(0, 0, 0, 0, 0);
    gdt_set_entry(1, 0, 0xFFFFF, 0x9A, 0xCF);
    gdt_set_entry(2, 0, 0xFFFFF, 0x92, 0xCF);
    gdt_set_entry(3, 0, 0xFFFFF, 0xF2, 0xCF);
    gdt_set_entry(4, 0, 0xFFFFF, 0xFA, 0xCF);
    let tss_addr = &TSS as *const _ as u64;
    let tss_limit = core::mem::size_of::<Tss>() - 1;
    gdt_set_entry(5, tss_addr, tss_limit as u64, 0x89, 0x00);
    gdt_set_entry(6, tss_addr >> 32, 0, 0, 0);
    TSS.rsp[0] = 0;
    TSS.rsp[1] = 0;
    TSS.rsp[2] = 0;
    TSS.ist[0] = 0;
    TSS.iomap_base = core::mem::size_of::<Tss>() as u16;
    gdt_load();
}

pub unsafe fn gdt_load() {
    core::arch::asm!("lgdt [{}]", in(reg) &GDT_PTR, options(nostack));
    core::arch::asm!("movw $0x10, %ax; movw %ax, %ds; movw %ax, %es; movw %ax, %fs; movw %ax, %gs", options(nostack));
    core::arch::asm!("ljmp $0x08, $1f; 1:", options(nostack));
}

pub unsafe fn tss_load() {
    core::arch::asm!("ltr %ax", in("ax") GDT_TSS, options(nostack));
}

pub unsafe fn tss_set_rsp0(rsp0: u64) {
    TSS.rsp[0] = rsp0;
}