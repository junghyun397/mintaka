mod text;
mod renderer;

use webp::Encoder;
use rusty_renju::notation::ffi::into_pos_slice;

#[repr(C)]
pub struct ByteBuffer {
    ptr: *mut u8,
    len: usize,
}

impl From<Vec<u8>> for ByteBuffer {
    fn from(value: Vec<u8>) -> Self {
        let boxed = value.into_boxed_slice();
        let len = boxed.len();
        let ptr = Box::into_raw(boxed) as *mut u8;

        Self { ptr, len }
    }
}

impl From<ByteBuffer> for Vec<u8> {
    fn from(value: ByteBuffer) -> Self {
        unsafe { Vec::from_raw_parts(value.ptr, value.len, value.len) }
    }
}

impl ByteBuffer {
    const NULL: Self = Self { ptr: std::ptr::null_mut(), len: 0 };
}


#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ImageFormat {
    Png = 1,
    Webp = 2,
}

impl TryFrom<u8> for ImageFormat {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Png),
            2 => Ok(Self::Webp),
            _ => Err(()),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_image_format_png() -> u8 { ImageFormat::Png as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_image_format_webp() -> u8 { ImageFormat::Webp as u8 }

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum HistoryRender {
    None = 0,
    Last = 1,
    Pair = 2,
    Sequence = 3,
}

impl TryFrom<u8> for HistoryRender {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Last),
            2 => Ok(Self::Pair),
            3 => Ok(Self::Sequence),
            _ => Err(()),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_image_renderer_none() -> u8 { HistoryRender::None as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_image_renderer_last() -> u8 { HistoryRender::Last as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_image_renderer_pair() -> u8 { HistoryRender::Pair as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_image_renderer_sequence() -> u8 { HistoryRender::Sequence as u8 }

pub struct RenderPayloads<'a> {
    pub history: rusty_renju::history::History,
    pub history_render: HistoryRender,
    pub offers: &'a [rusty_renju::notation::pos::Pos],
    pub blinds: &'a [rusty_renju::notation::pos::Pos],
    pub enable_forbidden: bool,
}

#[unsafe(no_mangle)]
pub fn rusty_renju_image_render(
    image_format: u8, webp_quality: f32,
    option: u8, enable_forbidden: bool,
    board: *const rusty_renju::board::Board,
    actions: *const u8, actions_len: usize,
    offers: *const u8, offers_len: usize,
    blinds: *const u8, blinds_len: usize,
) -> ByteBuffer {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(image_format) = ImageFormat::try_from(image_format)
        && let Ok(history_render) = HistoryRender::try_from(option)
        && let Some(actions) = rusty_renju::notation::ffi::from_raw_maybe_pos_slice(actions, actions_len)
        && let Some(offers) = rusty_renju::notation::ffi::from_raw_maybe_pos_slice(offers, offers_len).map(into_pos_slice)
        && let Some(blinds) = rusty_renju::notation::ffi::from_raw_maybe_pos_slice(blinds, blinds_len).map(into_pos_slice)
    {
        let history = actions.into();

        let pixmap = renderer::render_pixmap(&board, RenderPayloads {
            history, history_render, offers, blinds, enable_forbidden,
        });

        match image_format {
            ImageFormat::Png => pixmap.encode_png().unwrap().into(),
            ImageFormat::Webp => {
                let rgba = renderer::demultiply(&pixmap);

                Encoder::from_rgba(&rgba, pixmap.width(), pixmap.height())
                    .encode((webp_quality * 100.0).clamp(0.0, 100.0))
                    .to_vec()
                    .into()
            }
        }
    } else {
        ByteBuffer::NULL
    }
}

#[unsafe(no_mangle)]
pub fn rusty_renju_image_free_byte_buffer(byte_buffer: *const ByteBuffer) {
    if let Some(byte_buffer) = unsafe { byte_buffer.as_ref() }
        && !byte_buffer.ptr.is_null()
    {
        unsafe { drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(byte_buffer.ptr, byte_buffer.len))) }
    }
}
