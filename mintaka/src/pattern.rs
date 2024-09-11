use crate::formation::{FIVE, INV_THREE_OVERLINE, OPEN_FOUR};
use crate::notation::color::Color;
use crate::notation::pos::U_BOARD_WIDTH;
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

        let wall: u32 = !(!0 << (16 - self.length as u32));
        let bw = self.black_stones as u32 | wall;
        let ww = self.white_stones as u32 | wall;

        let mut acc: SlicePatch = EMPTY_SLICE_PATCH.clone();

        for offset in 0 ..= self.length as usize - 5 {
            find_patterns(
                &mut acc, offset,
                (self.black_stones >> offset) as u8, (self.white_stones >> offset) as u8,
                (bw >> offset) as u8, (ww >> offset) as u8,
            );
        }

        acc
    }

}

// O|X = equal_to, ! = not_equal_to, _ = patch, . = non-patch empty
#[allow(unused_variables)]
fn find_patterns(acc: &mut SlicePatch, offset: usize, b: u8, w: u8, bw: u8, ww:u8) {
    if pop_count_less_then_two!(b) && pop_count_less_then_two!(w) {
        return
    }

    let cold: u8 = !(bw | ww);

    let b_pop_count = b.count_ones();
    let w_pop_count = w.count_ones();

    macro_rules! match_pattern {
        ($packed:expr,$wall:expr,$equal_to:expr,$empty:expr,$not_equal_to:expr) => {
            ($packed & $equal_to) == $equal_to
                && (cold & $empty) == $empty
                && ($packed & $not_equal_to) == $not_equal_to
        };
        ($packed:expr,$equal_to:expr,$empty:expr) => {
            ($packed & $equal_to) == $equal_to
                && (cold & $empty) == $empty
        };
    }

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

    // OO

    // !.OO._.!
    // !.OO_.!

    // O.O

    // .O_O_.!

    // FOUR

    // !OO_O_!
    // !O.OO_!
    // !OOO._!
    // !O_O_O!
    // !OO__O!

    // OPEN-FOUR

    // !.OOO_.!
    if match_pattern!(b, ww, 0b0_00111000, 0b0_01000110, 0b0_10000001) {
        apply_patch_b!(5, OPEN_FOUR);
    }
    // !.OO_O.!

    // FIVE

    // !OO_OO!
    if match_pattern!(b, ww, 0b0_0110110, 0b0_0001000, 0b0_1000001) {
        apply_patch_b!(3, FIVE);
    }
    // !OOO_O!
    else if match_pattern!(b, ww, 0b0_0111010, 0b0_0000100, 0b0_1000001) {
        apply_patch_b!(4, FIVE);
    }
    // !O_OOO!
    else if match_pattern!(b, ww, 0b0_0101110, 0b0_0010000, 0b0_1000001){
        apply_patch_b!(3, FIVE);
    }
    // !OOOO_!
    else if match_pattern!(b, ww, 0b0_0111100, 0b0_0000010, 0b0_1000001) {
        apply_patch_b!(5, FIVE);
    }

    // XX_XX
    // XX.XX_
    // XXX_X
    // XXX.X_
    // X.XXX_
    // XXXX._

    // WIN

    if b & 0b00_11111 == 0b00_11111 {
        acc.winner = Some(Color::Black)
    }
    else if w & 0b00_11111 == 0b00_11111 {
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
