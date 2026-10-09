#![no_std]

use core::arch::asm;

#[no_mangle]
pub extern "C" fn kernel_early_init() {
    // early printk init would go here
}

#[no_mangle]
pub extern "C" fn kernel_main_rust(_multiboot_info: *mut u8, _magic: u32) {
    kernel_early_init();
    unsafe {
        gdt::gdt_init();
        gdt::tss_load();
        idt::idt_init();
        idt::irq_init();
    }
    loop { unsafe { asm!("hlt"); } }
}