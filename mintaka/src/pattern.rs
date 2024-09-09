use crate::formation::{FIVE, INV_THREE_OVERLINE};
use crate::notation::color::Color;
use crate::notation::rule::U_BOARD_WIDTH;
use crate::pop_count_less_then_two;
use crate::slice::Slice;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct FormationPatch {
    pub black_patch: u8,
    pub white_patch: u8
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct SlicePatch {
    pub patch: [FormationPatch; U_BOARD_WIDTH],
    pub winner: Option<Color>
}

pub const EMPTY_SLICE_PATCH: SlicePatch = SlicePatch {
    patch: [FormationPatch { black_patch: 0, white_patch: 0 }; U_BOARD_WIDTH],
    winner: None,
};

impl Slice {

    pub fn calculate_slice_patch(&self) -> SlicePatch {
        if pop_count_less_then_two!(self.black_stones) && pop_count_less_then_two!(self.white_stones) {
            return EMPTY_SLICE_PATCH
        }

        let wall: u32 = !(!0 << (16 - self.length));
        let bw = self.black_stones as u32 | wall;
        let ww = self.white_stones as u32 | wall;

        let mut acc: SlicePatch = EMPTY_SLICE_PATCH.clone();

        for offset in 0 .. self.length as usize - 5 {
            find_patterns(
                &mut acc, offset,
                (self.black_stones >> offset) as u8, (self.white_stones >> offset) as u8,
                (bw >> offset) as u8, (ww >> offset) as u8,
            )
        }

        acc
    }

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
    };
}

// O|X = equal_to, ^ = not_equal_to, _ = patch, . = non-patch empty
fn find_patterns(acc: &mut SlicePatch, offset: usize, b: u8, w: u8, bw: u8, ww:u8) {
    if pop_count_less_then_two!(b) && pop_count_less_then_two!(w) {
        return
    }

    let b_pop_count = b.count_ones();
    let w_pop_count = w.count_ones();

    macro_rules! apply_patch_b {
        ($p1:expr,$k1:expr) => {
            acc.patch[offset + $p1] = FormationPatch { black_patch: $k1, white_patch: 0 };
            return
        };
        ($p1:expr,$k1:expr,$p2:expr,$k2:expr) => {
            acc.patch[offset + $p1] = FormationPatch { black_patch: $k1, white_patch: 0 };
            acc.patch[offset + $p2] = FormationPatch { black_patch: $k2, white_patch: 0 };
            return
        };
    }

    macro_rules! apply_patch_w {
        ($p1:expr,$k1:expr) => {
            acc.patch[offset + $p1] = FormationPatch { black_patch: 0, white_patch:$k1 };
            return
        };
        ($p1:expr,$k1:expr,$p2:expr,$k2:expr) => {
            acc.patch[offset + $p1] = FormationPatch { black_patch: 0, white_patch:$k1 };
            acc.patch[offset + $p2] = FormationPatch { black_patch: 0, white_patch:$k2 };
            return
        };
    }

    // TODO: STRONG control hazard, needs optimization

    // THREE

    // FOUR

    // OPEN-FOUR

    // FIVE

    if match_pattern!(b, ww, 0b1111_0000, 0b0000_1000, 0b0000_0100) { // OOOO_^
        apply_patch_b!(4, FIVE);
    } else if match_pattern!(b, ww, 0b0000_1111, 0b0001_0000, 0b0010_0000) { // ^_OOOO
        apply_patch_b!(3, FIVE);
    } else if match_pattern!(w, bw, 0b1111_0000, 0b0000_1000) { // XXXX_
    } else if match_pattern!(w, bw, 0b0000_1111, 0b0001_0000) { // _XXXX
    }

    // WIN

    if b & 0b1111_1000 == 0b1111_1000 {
        acc.winner = Some(Color::Black)
    } else if w & 0b1111_1000 == 0b1111_1000 {
        acc.winner = Some(Color::White)
    }
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
