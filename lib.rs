#![no_std]
#![no_main]

pub mod gdt;
pub mod idt;
pub mod kernel;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}