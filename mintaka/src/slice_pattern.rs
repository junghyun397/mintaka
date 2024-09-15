use crate::notation::color::Color;
use crate::notation::pos::U_BOARD_WIDTH;
use crate::pattern::{CLOSED_FOUR_SINGLE, CLOSE_THREE, FIVE, OPEN_FOUR, OPEN_THREE};
use crate::pop_count_less_then_two;
use crate::slice::Slice;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct PatternPatch {
    pub black_patch: u8,
    pub white_patch: u8
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct SlicePatch {
    pub patch: [PatternPatch; U_BOARD_WIDTH],
    pub winner: Option<Color>
}

pub const EMPTY_SLICE_PATCH: SlicePatch = SlicePatch {
    patch: [PatternPatch { black_patch: 0, white_patch: 0 }; U_BOARD_WIDTH],
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

fn find_patterns(acc: &mut SlicePatch, offset: usize, b: u8, w: u8, bw: u8, ww:u8) {
    if pop_count_less_then_two!(b) && pop_count_less_then_two!(w) {
        return
    }

    let cold: u8 = !(bw | ww);

    let b_pop_count = b.count_ones();
    let w_pop_count = w.count_ones();

    /*
    # PATTERN-DSL

    ## PATTERN-MATCH-LITERAL:
    * O = self-color-hot
    * X = reversed-color-hot
    * ! = not self-color-hot
    * . = cold

    > EX: match black's closed-four = "!OOO..!"

    ## PATTERN-PATCH_LITERAL:
    * 3 = open-three
    * C = close-three
    * 4 = open-four
    * F = closed-four-single
    * 5 = five

    EX: match black's closed-four = "!OOO.F"
    */

    macro_rules! match_pattern {
        ($pattern:literal,$packed:expr,$wall:expr,rev=$rev:expr) => {{
            const equal_to: u8 = parse_pattern_literal('O', $pattern, $rev);
            const not_equal_to: u8 = parse_pattern_literal('!', $pattern, $rev);
            const empty: u8 = parse_pattern_literal('.', $pattern, $rev);
            const block: u8 = parse_pattern_literal('X', $pattern, $rev);

            $packed & equal_to == equal_to
                && $packed & not_equal_to == 0
                && cold & empty == empty
                && $wall & block == block
        }};
    }

    macro_rules! match_pattern_b {
        ($pattern:literal) => (match_pattern_b!($pattern, rev=false));
        ($pattern:literal,rev=$rev:expr) => (match_pattern!($pattern, b, ww, rev=$rev));
    }

    macro_rules! match_pattern_w {
        ($pattern:literal) => (match_pattern_w!($pattern, rev=false));
        ($pattern:literal,rev=$rev:expr) => (match_pattern!($pattern, w, bw, rev=$rev));
    }

    macro_rules! apply_patch_b {
        ($patch:literal) => {
            apply_patch_b!($patch,rev=false);
        };
        ($patch:literal,rev=$rev:expr) => {{
            const pos_kind: (usize, u8) = parse_patch_literal($patch, $rev);

            // branch removed at compile time
            if (pos_kind.1 == CLOSED_FOUR_SINGLE) {
                let original = acc.patch[offset + pos_kind.0].black_patch;
                acc.patch[offset + pos_kind.0] = PatternPatch {
                    black_patch: increase_closed_four(original),
                    white_patch: 0
                };
            } else {
                acc.patch[offset + pos_kind.0] = PatternPatch {
                    black_patch: pos_kind.1,
                    white_patch: 0
                };
            }
        }};
    }

    macro_rules! apply_patch_w {
        ($patch:literal) => {
            apply_patch_w!($patch,rev=false);
        };
        ($patch:literal,rev=$rev:expr) => {{
            const pos_kind: (usize, u8) = parse_patch_literal($patch, $rev);

            // branch removed at compile time
            if (pos_kind.1 == CLOSED_FOUR_SINGLE) {
                let original = acc.patch[offset + pos_kind.0].white_patch;
                acc.patch[offset + pos_kind.0] = PatternPatch {
                    white_patch: increase_closed_four(original),
                    black_patch: 0
                };
            } else {
                acc.patch[offset + pos_kind.0] = PatternPatch {
                    white_patch: pos_kind.1,
                    black_patch: 0
                };
            }
        }};
    }

    // TODO: STRONG control hazard, needs optimization.

    // THREE

    // OO

    // !.OO._.!
    // !.OO_.!

    // O.O

    // .O_O_.!

    // FOUR

    if match_pattern_b!("!OO.O.!") {
        apply_patch_b!("!OO.OF!");
        return
    }
    if match_pattern_b!("XOO.O.!") {
        apply_patch_b!("XOOFO.!");
        return
    }
    if match_pattern_b!("!OO..O!") {
        apply_patch_b!("!OOF.O!");
        apply_patch_b!("!OO.FO!");
        return
    }
    if match_pattern_b!("!O.OO.!") {
        apply_patch_b!("!O.OOF!");
        return
    }
    if match_pattern_b!("!OOO..!") {
        apply_patch_b!("!OOO.F!");
        return
    }
    if match_pattern_b!("!O.O.O!") {
        apply_patch_b!("!OFO.O!");
        apply_patch_b!("!O.OFO!");
        return
    }
    // !XX__X!

    // OPEN-FOUR

    // !.XXX_.!
    // !.XX_X.!
    // !_XX.X_!

    // CLOSE-THREE

    // O.XXX._!

    // FIVE

    if match_pattern_b!("!OO.OO!") {
        apply_patch_b!("!OO5OO!");
        return
    }
    if match_pattern_b!("!OOO.O!") {
        apply_patch_b!("!OOO5O!");
        return
    }
    if match_pattern_b!("!OOOO.!") {
        apply_patch_b!("!OOOO5!");
        return
    }

    if match_pattern_w!("OO_OO") {
        apply_patch_w!("OO5OO");
        return
    }
    if match_pattern_w!("OOO.O") {
        apply_patch_w!("OOO5O");
        return
    }
    if match_pattern_w!("OOOO.") {
        apply_patch_w!("OOOO5");
        return
    }

    // WIN

    if b & 0b000_11111 == 0b000_11111 {
        acc.winner = Some(Color::Black);
        return;
    }
    if w & 0b000_11111 == 0b000_11111 {
        acc.winner = Some(Color::White);
        return;
    }
}

fn has_overline(packed: u16) -> bool {
    let mut packed = packed;

    packed &= packed >> 1; // make space for 3-bits shift
    packed &= packed >> 1; // make space for 3-bits shift
    packed &= packed >> 3; // 6 - 1 - 1 - 3 = 1

    packed != 0
}

fn increase_closed_four(packed: u8) -> u8 {
    packed | (0b1000_000 >> (packed >> 7))
}

const fn parse_pattern_literal(kind: char, source: &str, reversed: bool) -> u8 {
    let mut acc: u8 = 0;

    let mut idx: usize = 0;
    while idx < source.len() {
        if source.as_bytes()[idx] as char == kind {
            acc |= 0b1 << idx;
        }

        idx += 1;
    }

    acc
}

const fn parse_patch_literal(source: &str, reverse: bool) -> (usize, u8) {
    let mut idx: usize = 0;
    while idx < source.len() {
        match source.as_bytes()[idx] as char {
            '3' => return (idx, OPEN_THREE),
            'C' => return (idx, CLOSE_THREE),
            '4' => return (idx, OPEN_FOUR),
            'F' => return (idx, CLOSED_FOUR_SINGLE),
            '5' => return (idx, FIVE),
            _ => {}
        }
        idx += 1;
    }

    unreachable!()
}
