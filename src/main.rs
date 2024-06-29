#![no_std]
#![no_main]
use core::arch::asm;
use core::panic::PanicInfo;
use core::num::Wrapping;

fn panik() -> ! {
	loop {unsafe {asm!("hlt");}}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	loop {unsafe {asm!("hlt");}}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    //let foo: Wrapping<usize> = Wrapping::new(0);
    panik();
}
