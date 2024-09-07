use crate::formation::{FIVE, INV_THREE_OVERLINE};
use crate::notation::rule::U_BOARD_WIDTH;
use crate::slice::Slice;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct FormationPatch {
    pub black_patch: u8,
    pub white_patch: u8
}

// 240-bit
pub type SlicePatch = [FormationPatch; U_BOARD_WIDTH];

pub const EMPTY_SLICE_PATCH: SlicePatch = [FormationPatch { black_patch: 0, white_patch: 0}; U_BOARD_WIDTH];

impl Slice {

    pub fn calculate_slice_patch(&self) -> SlicePatch {
        let wall: u16 = !(!0 << (16 - self.length));
        EMPTY_SLICE_PATCH
    }

}

macro_rules! fp {
    ($b:expr,$w:expr) => {
        FormationPatch {
            black_patch: $b,
            white_patch: $w
        }
    };
}

macro_rules! match_pattern {
    ($packed:expr,$wall:expr,$equal_to:expr,$empty:expr,$not_equal_to:expr) => {
        ($packed & $equal_to) == $equal_to
            && (($packed | $wall) & $empty) != $empty
            && ($packed & $not_equal_to) == $not_equal_to
    };
    ($packed:expr,$wall:expr,$equal_to:expr,$empty:expr) => {
        ($packed & $equal_to) == $equal_to
            && (($packed | $wall) & $empty) != $empty
    }
}

// O|X = equal_to, ^ = not_equal_to, _ = patch, . = non-patch empty
fn find_patterns(acc: &mut SlicePatch, offset: usize, b: u8, w: u8, bw: u8, ww:u8) {
    if b == 0 || w == 0 {
        return
    }

    // FIVE
    if match_pattern!(b, ww, 0b1111_0000, 0b0000_1000, 0b0000_0100) { // OOOO_^
        acc[offset + 4] = fp!(FIVE, 0)
    } else if match_pattern!(b, ww, 0b0000_1111, 0b0001_0000, 0b0010_0000) { // ^_OOOO
        acc[offset + 3] = fp!(FIVE, 0)
    } else if match_pattern!(w, bw, 0b1111_0000, 0b0000_1000) { // XXXX_
        acc[offset + 4] = fp!(0, FIVE)
    } else if match_pattern!(w, bw, 0b0000_1111, 0b0001_0000) { // _XXXX
        acc[offset + 3] = fp!(0, FIVE)
    }

    todo!()
}

fn increase_closed_four(packed: u8) -> u8 {
    packed | (0b1000_000 >> (packed >> 7))
}

fn mark_blind_three(packed: u8) -> u8 {
    packed | INV_THREE_OVERLINE
}

fn unmark_blind_three(packed: u8) -> u8 {
    packed & !INV_THREE_OVERLINE
}
