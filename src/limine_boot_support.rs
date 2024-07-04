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
