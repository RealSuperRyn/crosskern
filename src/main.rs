#![no_std]
#![no_main]
#![allow(unused_imports, dead_code)] //These are obnoxious.
#![feature(
    const_trait_impl,
    const_option,
    sync_unsafe_cell,
    c_size_t,
    concat_bytes
)]
use core::concat_bytes;
use core::panic::PanicInfo;
use core::{arch::asm, ffi::c_int}; //we avoid core::fmt like it's the plague

use crosshw::boot::FrameBuf;
use lazy_static::lazy_static;

use core::ffi::{c_size_t, c_void};
pub mod debugging;
use self::debugging::*;
pub mod limine_boot_support;
use self::limine_boot_support::*;

fn panik() -> ! {
    //We halt and catch fire.
    unsafe { asm!("cli") } //stop interrupts from interrupting our indefinite fire catching session
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn undefined_behavior(_info: &PanicInfo) -> ! {
    //This should never trigger, but it's included otherwise the compiler will whine relentlessly.
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let fb = unsafe { &*FRAMEBUFFER.lock().get() }; //Until we initiate multithreading, this is fine to keep.
    let mut p = Printer::new(fb);
    unsafe { fb.fill(0x000088FF) }
    p.print(
        concat!(
            "Cross Kernel ",
            env!("CARGO_PKG_VERSION"),
            " -> Entered entry function\n",
            "License: ",
            include_str!("../LICENSE")
        )
        .as_bytes(),
        fb,
        0xFFFFFFFF,
        0x000088FF,
    );

    main_loop();
}

pub fn main_loop() -> ! {
    loop {
        panik();
    }
}
