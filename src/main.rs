#![no_std]
#![no_main]
#![allow(unused_imports, dead_code)] //These are obnoxious.
#![feature(const_trait_impl, const_option, sync_unsafe_cell, c_size_t)]
use core::panic::PanicInfo;
use core::{arch::asm, ffi::c_int};

use crosshw::boot::FrameBuf;
use lazy_static::lazy_static;

use core::ffi::{c_size_t, c_void};
//If I don't implement these, the linker will scream at me for not having them
/*
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut c_void, src: *const c_void, n: c_size_t) -> *mut c_void {
    let pdest: *mut u8 = dest as *mut u8;
    let psrc: *const u8 = src as *const u8;
    for i in 0..n {
        unsafe { *pdest.add(i) = *psrc.add(i) }
    }
    return dest;
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut c_void, c: c_int, n: c_size_t) -> *mut c_void {
    let p: *mut u8 = s as *mut u8;
    for i in 0..n {
        *p.add(i) = c as u8;
    }
    return s;
}
*/
//#[cfg(feature = "debugger")]
mod debugging {
    use core::cell::SyncUnsafeCell;
    use crosshw::boot::FrameBuf;
    //We use a Dual Color Image to store the font (If you haven't heard of it, that's because it was made specifically for this.)
    pub const FONT_RAW: &'static [u8] = include_bytes!("../font.dci");
    pub const FONT_LEN: usize = FONT_RAW.len();
    pub const FONT: DCIImage = DCIImage {
        width: u32::from_ne_bytes(
            *FONT_RAW
                .split_first_chunk::<8>()
                .unwrap()
                .0
                .split_first_chunk::<4>()
                .unwrap()
                .0,
        ),
        height: u32::from_ne_bytes(
            *FONT_RAW
                .split_first_chunk::<8>()
                .unwrap()
                .0
                .split_last_chunk::<4>()
                .unwrap()
                .1,
        ),
        pixels: FONT_RAW.split_last_chunk::<FONT_LEN>().unwrap().1,
    };
    pub const FONT_WIDTH: usize = 128;
    pub const FONT_HEIGHT: usize = 112;
    fn get_character(character: u8) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        let first_pixel_indices = DCIImage::get_index(
            (character / 16) as usize * FONT_WIDTH * 8 + (character as usize * 8),
        );
        let mut j = 0;
        for i in bytes.iter_mut() {
            *i = FONT.pixels[first_pixel_indices.0 + 16 * j];
            j += 1;
        }
        bytes
    }
    fn blit_char(character: u8, fb: &FrameBuf, fgcolor: u32, bgcolor: u32, x: usize, y: usize) {
        for i in get_character(character).iter() {
            for j in 0..7 {
                let col = if i & !(1 << j) != 0 { fgcolor } else { bgcolor };
                unsafe {
                    fb.set_pixel(col, x, y);
                }
            }
        }
    }
    pub fn print(chars: &[u8], fb: &FrameBuf, fgcolor: u32, bgcolor: u32, x: usize, y: usize) {
        let mut px = x;
        let mut py = y;
        for i in chars.iter() {
            if px + 8 > fb.mode.width as usize {
                px = 0;
                py += 8;
            }
            if py + 8 > fb.mode.height as usize {
                px = 0;
                py = 0;
            }
            blit_char(*i, fb, fgcolor, bgcolor, px, py);
            px += 8;
        }
    }
    pub struct DCIImage {
        pub width: u32,
        pub height: u32,
        pub pixels: &'static [u8],
    }
    impl DCIImage {
        pub fn new(width: u32, height: u32) -> DCIImage {
            DCIImage {
                width: width,
                height: height,
                pixels: &[0u8; 128 * 112],
            }
        }
        #[inline(always)] //See below comment.
        pub fn get_index(full_idx: usize) -> (usize, u8) {
            ((full_idx & (!7)) >> 3, (full_idx & 7) as u8)
        }
    }
}
//#[cfg(feature = "debugger")]
use self::debugging::*;
//#[cfg(feature = "limine")]
mod limine_boot_support {
    use core::cell::SyncUnsafeCell;
    use crosshw::boot::*;
    use lazy_static::lazy_static;
    use limine::{
        request::{FramebufferRequest, PagingModeRequest, RsdpRequest},
        BaseRevision,
    };
    use spin;
    #[used]
    #[link_section = ".requests"]
    pub static BASE_REVISION: BaseRevision = BaseRevision::new();
    #[used]
    #[link_section = ".requests"]
    pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
    #[used]
    #[link_section = ".requests"]
    pub static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();
    #[used]
    #[link_section = ".requests"]
    pub static PAGING_REQUEST: PagingModeRequest =
        PagingModeRequest::new().with_mode(limine::paging::Mode::FOUR_LEVEL);
    lazy_static! {
        pub static ref FRAMEBUFFER: spin::Mutex::<SyncUnsafeCell<FrameBuf>> = {
            let FRAMEBUFFER_RAW = FRAMEBUFFER_REQUEST
                .get_response()
                .unwrap()
                .framebuffers()
                .next()
                .unwrap();

            spin::Mutex::<SyncUnsafeCell<FrameBuf>>::new(SyncUnsafeCell::<FrameBuf>::new(
                crosshw::boot::FrameBuf {
                    fb: FRAMEBUFFER_RAW.addr() as u64,
                    model: FBModel::RGB,
                    mode: FBMode {
                        bitsperpixel: FRAMEBUFFER_RAW.bpp(),
                        width: FRAMEBUFFER_RAW.width(),
                        height: FRAMEBUFFER_RAW.height(),
                    },
                },
            ))
        };
    }
    pub const STACK_SIZE: u64 = 0x10000; //FIXME: Enough for kernel, but kernel extensions may overrun the stack.
}

//#[cfg(feature = "limine")]
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
    //#[cfg(feature = "debugger")]
    loop {
        self::debugging::print(
            b"Is this thing on?",
            unsafe { &*FRAMEBUFFER.lock().get() },
            0xFFFFFFFF,
            0x00000000,
            0,
            0,
        );
    }
    main_loop();
}

pub fn main_loop() -> ! {
    loop {
        panik();
    }
}
