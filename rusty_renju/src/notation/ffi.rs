use crate::notation::pos::{MaybePos, Pos};

pub fn from_raw_maybe_pos_slice<'a>(slice: *const u8, len: usize) -> Option<&'a [MaybePos]> {
    if len == 0 {
        return Some(&[]);
    }

    if slice.is_null() {
        return None;
    }

    Some(unsafe { std::slice::from_raw_parts(slice as *const MaybePos, len) })
}

pub fn from_raw_pos_slice<'a>(slice: *const u8, len: usize) -> Option<&'a [Pos]> {
    if len == 0 {
        return Some(&[]);
    }

    if slice.is_null() {
        return None;
    }

    Some(unsafe { std::slice::from_raw_parts(slice as *const Pos, len) })
}
