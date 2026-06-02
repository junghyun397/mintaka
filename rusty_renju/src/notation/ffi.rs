use crate::board::Board;
use crate::notation::pos::{MaybePos, Pos};
use crate::notation::rule::RuleKind;

#[repr(C)]
pub struct CBoard {
    pub inner: Board<{ RuleKind::Renju }>,
}

impl From<Board<{ RuleKind::Renju }>> for CBoard {
    fn from(value: Board<{ RuleKind::Renju }>) -> Self {
        Self { inner: value }
    }
}

pub fn from_raw_maybe_pos_slice<'a>(slice: *const u8, len: usize) -> Option<&'a [MaybePos]> {
    if len == 0 {
        return Some(&[]);
    }

    if slice.is_null() {
        return None;
    }

    Some(unsafe { std::slice::from_raw_parts(slice as *const MaybePos, len) })
}

pub fn into_pos_slice(maybe_pos_slice: &[MaybePos]) -> &[Pos] {
    unsafe { std::mem::transmute::<&[MaybePos], &[Pos]>(maybe_pos_slice) }
}
