use crate::formation::{CLOSE_THREE, FIVE, INV_THREE_OVERLINE, OPEN_FOUR};
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
        ($pos:expr,$patch:expr,$($rest:tt)*) => {
            acc.patch[offset + $pos] = FormationPatch { black_patch: $patch, white_patch: 0 };
            apply_patch_b!($($rest)*);
        };
        ($pos:expr, INC_CLOSED_FOUR,$($rest:tt)*) => {
            let original = acc.patch[offset + $p1];
            acc.patch[offset + $p1] = FormationPatch {
                black_patch: increase_closed_four(original.black_patch)
                white_patch: 0,
            };
            apply_patch_b!($($rest)*);
        };
        () => {
            return
        }
    }

    macro_rules! apply_patch_w {
        ($pos:expr,$patch:expr,$($rest:tt)*) => {
            acc.patch[offset + $pos] = FormationPatch { black_patch: $patch, white_patch: 0 };
            apply_patch_b!($($rest)*);
        };
        ($pos:expr,INC_CLOSED_FOUR,$($rest:tt)*) => {
            let original = acc.patch[offset + $p1];
            acc.patch[offset + $p1] = FormationPatch {
                black_patch: increase_closed_four(original.black_patch)
                white_patch: 0,
            };
            apply_patch_b!($($rest)*);
        };
        () => {
            return
        }
    }

    // TODO: STRONG control hazard

    // THREE

    if b_pop_count < 2 {
        return
    }

    // OO

    // !.OO._.!
    // !.OO_.!

    // O.O

    // .O_O_.!

    if w_pop_count < 2 {
        return
    }

    // FOUR

    if b_pop_count < 3 {
        return
    }

    // !XX_X_!
    // !X.XX_!
    // !XXX._!
    // !X_X_X!
    if match_pattern!(b, ww, 0b0_0101010, 0b0_0010100, 0b0_1000001) {
        apply_patch_w!(2, OPEN_FOUR, 4, OPEN_FOUR,);
    }
    // !XX__X!

    if w_pop_count < 3 {
        return
    }

    // OPEN-FOUR

    // !.XXX_.!
    if match_pattern!(b, ww, 0b0_00111000, 0b0_01000110, 0b_10000001) {
        apply_patch_b!(5, OPEN_FOUR, 5, CLOSE_THREE,);
    }
    // !.XX_X.!
    // !_XX.X_!
    if match_pattern!(b, ww, 0b0_00110100, 0b0_01001010, 0b_1000001) {
        apply_patch_b!(4, OPEN_FOUR,);
        apply_patch_b!(1, CLOSE_THREE, 5, CLOSE_THREE,);
    }

    // CLOSE-THREE

    // O.XXX._!

    // FIVE

    if b_pop_count < 4 {
        return
    }

    // !XX_XX!
    else if match_pattern!(b, ww, 0b0_0110110, 0b0_0001000, 0b0_1000001) {
        apply_patch_b!(3, FIVE,);
    }
    // !XXX_X!
    else if match_pattern!(b, ww, 0b0_0111010, 0b0_0000100, 0b0_1000001) {
        apply_patch_b!(4, FIVE,);
    }
    // !XXXX_!
    else if match_pattern!(b, ww, 0b0_0111100, 0b0_0000010, 0b0_1000001) {
        apply_patch_b!(5, FIVE,);
    }

    if w_pop_count < 4 {
        return
    }

    // OO_OO
    // OOO_O
    // OOO.O_
    // O.OOO_

    // WIN

    if b & 0b00_11111 == 0b00_11111 {
        acc.winner = Some(Color::Black)
    }
    else if w & 0b00_11111 == 0b00_11111 {
        acc.winner = Some(Color::White)
    }
}

fn has_overline(packed: u16) -> bool {
    let mut packed = packed;

    packed &= packed >> 1; // make space for shift
    packed &= packed >> 1; // make space for shift
    packed &= packed >> 3; // 6 - 1 - 1 - 3 = 1

    packed != 0
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
