#![no_std]

pub const IDT_ENTRIES: usize = 256;
pub const IDT_INT_GATE: u8 = 0x8E;
pub const IDT_TRAP_GATE: u8 = 0x8F;
pub const IDT_USER: u8 = 0x60;

#[repr(C, packed)]
pub struct IdtEntry {
    pub offset_low: u16,
    pub selector: u16,
    pub ist: u8,
    pub type_attr: u8,
    pub offset_mid: u16,
    pub offset_high: u32,
    pub zero: u32,
}

#[repr(C, packed)]
pub struct IdtPtr {
    pub limit: u16,
    pub base: u64,
}

#[repr(C, packed)]
pub struct InterruptFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

pub type IsrHandler = extern "C" fn(frame: *mut InterruptFrame);

static mut IDT: [IdtEntry; IDT_ENTRIES] = [IdtEntry {
    offset_low: 0, selector: 0, ist: 0, type_attr: 0,
    offset_mid: 0, offset_high: 0, zero: 0,
}; IDT_ENTRIES];

static mut IDT_PTR: IdtPtr = IdtPtr { limit: 0, base: 0 };
static mut ISR_HANDLERS: [Option<IsrHandler>; IDT_ENTRIES] = [None; IDT_ENTRIES];

pub unsafe fn idt_set_entry(index: usize, handler: u64, selector: u16, type_attr: u8, ist: u8) {
    if index >= IDT_ENTRIES { return; }
    let entry = &mut IDT[index];
    entry.offset_low = handler as u16;
    entry.selector = selector;
    entry.ist = ist;
    entry.type_attr = type_attr;
    entry.offset_mid = (handler >> 16) as u16;
    entry.offset_high = (handler >> 32) as u32;
    entry.zero = 0;
}

pub unsafe fn idt_init() {
    IDT_PTR.limit = core::mem::size_of::<IdtEntry>() * IDT_ENTRIES as usize - 1;
    IDT_PTR.base = &IDT as *const _ as u64;
    for i in 0..IDT_ENTRIES { idt_set_entry(i, 0, 0, 0, 0); }
    idt_load();
}

pub unsafe fn idt_load() {
    core::arch::asm!("lidt [{}]", in(reg) &IDT_PTR, options(nostack));
}

pub unsafe fn register_isr(vector: usize, handler: IsrHandler) {
    if vector < IDT_ENTRIES { ISR_HANDLERS[vector] = Some(handler); }
}

pub unsafe fn register_irq(irq: usize, handler: IsrHandler) {
    register_isr(32 + irq, handler);
}

pub unsafe extern "C" fn isr_common_handler(frame: *mut InterruptFrame) {
    let vector = (*frame).rip & 0xFF;
    if vector < IDT_ENTRIES as u64 {
        if let Some(handler) = ISR_HANDLERS[vector as usize] {
            handler(frame);
        } else {
            loop { core::arch::asm!("hlt"); }
        }
    }
}

pub unsafe fn irq_init() {
    outb(0x20, 0x11); outb(0xA0, 0x11);
    outb(0x21, 0x20); outb(0xA1, 0x28);
    outb(0x21, 0x04); outb(0xA1, 0x02);
    outb(0x21, 0x01); outb(0xA1, 0x01);
    outb(0x21, 0x00); outb(0xA1, 0x00);
}

pub unsafe fn irq_enable(irq: usize) {
    let port = if irq < 8 { 0x21 } else { 0xA1 };
    let irq_num = if irq < 8 { irq } else { irq - 8 };
    let mask = inb(port); outb(port, mask & !(1 << irq_num));
}

pub unsafe fn irq_disable(irq: usize) {
    let port = if irq < 8 { 0x21 } else { 0xA1 };
    let irq_num = if irq < 8 { irq } else { irq - 8 };
    let mask = inb(port); outb(port, mask | (1 << irq_num));
}

pub unsafe fn irq_eoi(irq: usize) {
    if irq >= 8 { outb(0xA0, 0x20); }
    outb(0x20, 0x20);
}

#[inline(always)] unsafe fn inb(port: u16) -> u8 {
    let ret: u8; core::arch::asm!("inb {}, {}", out(reg_al) ret, in("Nd") port); ret
}

#[inline(always)] unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("outb {}, {}", in("al") val, in("Nd") port);
}