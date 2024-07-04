use core::cell::SyncUnsafeCell;
use crosshw::boot::FrameBuf;
use lazy_static::lazy_static;
//We use a Dual Color Image to store the font (If you haven't heard of it, that's because it was made specifically for this.)
pub const FONT_RAW: &'static [u8] = include_bytes!("../font.dci");
pub const FONT_LEN: usize = FONT_RAW.len();
lazy_static! {
    pub static ref FONT: DCIImage = {
        let raw_font = include_bytes!("../font.dci");
        let width = u32::from_ne_bytes((raw_font[0], raw_font[1], raw_font[2], raw_font[3]).into());
        let height =
            u32::from_ne_bytes((raw_font[4], raw_font[5], raw_font[6], raw_font[7]).into());
        DCIImage {
            width: width,
            height: height,
            pixels: &raw_font[8..],
        }
    };
}
pub const FONT_WIDTH: usize = 128;
pub const FONT_HEIGHT: usize = 112;
fn get_character(c: u8) -> [u8; 8] {
    if c <= 32 {
        return [0u8; 8];
    }
    let character = c - 32;
    let mut bytes = [0u8; 8];
    let first_pixel_indices = DCIImage::get_index(
        (character / 16) as usize * FONT_WIDTH * 8 + ((character as usize & 15) * 8),
    );
    let mut j = 0;
    for i in bytes.iter_mut() {
        *i = FONT.pixels[first_pixel_indices.0 + 16 * j];
        j += 1;
    }
    bytes
}
pub fn blit_char(character: u8, fb: &FrameBuf, fgcolor: u32, bgcolor: u32, x: usize, y: usize) {
    let mut py = y;
    for i in get_character(character).iter() {
        for j in 0..8 {
            let col = if i & (1 << j) != 0 { fgcolor } else { bgcolor };
            unsafe {
                fb.set_pixel(col, x + j, py);
            }
        }
        py += 1;
    }
}
pub fn print(
    chars: &[u8],
    fb: &FrameBuf,
    fgcolor: u32,
    bgcolor: u32,
    x: usize,
    y: usize,
) -> (usize, usize) {
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
        if *i == 0x0a {
            py += 8;
            px = 0
        } else {
            blit_char(*i, fb, fgcolor, bgcolor, px, py);
            px += 8;
        }
    }
    (px, py)
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
pub struct Printer {
    cx: usize,
    cy: usize,
}
impl Printer {
    pub fn new(fb: &FrameBuf) -> Printer {
        unsafe { fb.fill(0x00000000) }
        Printer { cx: 0, cy: 0 }
    }
    pub fn print(&mut self, chars: &[u8], fb: &FrameBuf, fgcolor: u32, bgcolor: u32) {
        let indices = self::print(chars, fb, fgcolor, bgcolor, self.cx, self.cy);
        if indices.1 < self.cy {
            unsafe { fb.fill(0x00000000) }
            self.cx = 0;
            self.cy = 0;
        } //If the print function wrapped around, black the screen out.
    }
}
