use crate::notation::color::Color;
use crate::notation::pos::U_BOARD_WIDTH;
use crate::pattern::{CLOSED_FOUR_SINGLE, CLOSE_THREE, FIVE, INV_THREE_OVERLINE, OPEN_FOUR, OPEN_THREE};
use crate::pop_count_less_then_two;
use crate::slice::Slice;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct PatternPatch {
    pub black_patch: u8,
    pub white_patch: u8
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct SlicePattern {
    pub patch: [PatternPatch; U_BOARD_WIDTH],
    pub winner: Option<Color>
}

pub const EMPTY_SLICE_PATCH: SlicePattern = SlicePattern {
    patch: [PatternPatch { black_patch: 0, white_patch: 0 }; U_BOARD_WIDTH],
    winner: None,
};

impl Slice {

    pub fn calculate_slice_patch(&self) -> SlicePattern {
        if pop_count_less_then_two!(self.black_stones) && pop_count_less_then_two!(self.white_stones) {
            return EMPTY_SLICE_PATCH
        }

        // padding = 3
        let wall: u32 = !(!(u32::MAX << self.length as u32) << 3);
        let b: u32 = (self.black_stones as u32) << 3;
        let w: u32 = (self.white_stones as u32) << 3;
        let bw = b | wall;
        let ww = w | wall;
        let cold = !(bw | ww);

        let mut acc: SlicePattern = EMPTY_SLICE_PATCH;
        for shift in 0 ..= self.length as usize + 1 { // length - 5 + 3 * 2
            let cold_frag = (cold >> shift) as u8;
            if !(pop_count_less_then_two!(b) && pop_count_less_then_two!(w)) && cold != 0 {
                find_patterns(
                    &mut acc, shift as isize - 3,
                    b,
                    (b >> shift) as u8, (w >> shift) as u8,
                    (bw >> shift) as u8, (ww >> shift) as u8,
                    cold_frag
                );
            }
        }

        acc
    }

}

#[allow(clippy::too_many_arguments)]
fn find_patterns(acc: &mut SlicePattern, offset: isize, ob: u32, b: u8, w: u8, bw: u8, ww: u8, cold: u8) {
    /*
    # PATTERN-DSL

    ## PATTERN-MATCH-LITERAL
    * O = self-color-hot
    * X = reversed-color-hot
    * ! = not self-color-hot
    * . = cold

    > EX: match black's closed-four = "!OOO..!"

    ## PATTERN-PATCH_LITERAL
    * 3 = open-three
    * C = close-three
    * 4 = open-four
    * F = closed-four-single
    * 5 = five
    * 6 = overline

    EX: match black's closed-four = "!OOO.F"
    */

    let b_u32_vector: u32 = b as u32 | (b as u32) << 8 | (cold as u32) << 16 | (ww as u32) << 24;
    let w_u32_vector: u32 = w as u32 | (w as u32) << 8 | (cold as u32) << 16 | (bw as u32) << 24;

    macro_rules! match_long_pattern_for_black {
        (left, rev) => (match_long_pattern_for_black!(right));
        (right, rev) => (match_long_pattern_for_black!(left));
        (left) => {(
            ob & 0b1 << offset + 1 == 0 // offset + 3 - 1
        )};
        (right) => {(
            ob & 0b1 << offset + 11 == 0 // offset + 3 + 8 + 1
        )};
    }

    macro_rules! match_pattern {
        ($color:ident,$pattern:literal) => (match_pattern!($color, rev=false, $pattern));
        (black,rev=$rev:expr,$pattern:literal) => (match_pattern!(rev=$rev, $pattern, b_u32_vector));
        (white,rev=$rev:expr,$pattern:literal) => (match_pattern!(rev=$rev, $pattern, w_u32_vector));

        (rev=$rev:expr,$pattern:literal,$elements:expr) => {{
            const MASK: u32 = build_pattern_mask($pattern, $rev);
            const RESULT: u32 = build_pattern_result($pattern, $rev);

            $elements & MASK == RESULT
        }};
    }

    macro_rules! apply_patch {
        (black,rev=$rev:expr,$patch:literal) => {{
            const POS_KIND_TUPLE: (isize, u8) = parse_patch_literal($patch, $rev);

            let original = acc.patch[(offset + POS_KIND_TUPLE.0) as usize].black_patch;
            // branch removed at compile time
            if (POS_KIND_TUPLE.1 == CLOSED_FOUR_SINGLE) {
                acc.patch[(offset + POS_KIND_TUPLE.0) as usize] = PatternPatch {
                    black_patch: increase_closed_four(original),
                    white_patch: 0
                };
            } else {
                acc.patch[(offset + POS_KIND_TUPLE.0) as usize] = PatternPatch {
                    black_patch: original | POS_KIND_TUPLE.1,
                    white_patch: 0
                };
            }
        }};
        (white,rev=$rev:expr,$patch:literal) => {{
            const POS_KIND_TUPLE: (isize, u8) = parse_patch_literal($patch, $rev);

            let original = acc.patch[(offset + POS_KIND_TUPLE.0) as usize].white_patch;
            // branch removed at compile time
            if (POS_KIND_TUPLE.1 == CLOSED_FOUR_SINGLE) {
                acc.patch[(offset + POS_KIND_TUPLE.0) as usize] = PatternPatch {
                    black_patch: 0,
                    white_patch: increase_closed_four(original)
                };
            } else {
                acc.patch[(offset + POS_KIND_TUPLE.0) as usize] = PatternPatch {
                    black_patch: 0,
                    white_patch: original | POS_KIND_TUPLE.1
                };
            }
        }}
    }

    macro_rules! process_pattern {
        ($color:ident,symmetry,$pattern:literal,$($patch:literal),+) => {
            process_pattern!($color, rev=false, $pattern, $($patch),+);
        };
        ($color:ident,asymmetry,$pattern:literal,$($patch:literal),+) => {
            process_pattern!($color, rev=false, $pattern, $($patch),+);
            process_pattern!($color, rev=true, $pattern, $($patch),+)
        };
        (black,asymmetry,long-pattern,$position:ident,$pattern:literal,$($patch:literal),+) => {
            if match_pattern!(black, rev=false, $pattern) && match_long_pattern_for_black!($position) {
                $(
                    apply_patch!(black, rev=false, $patch);
                )*
            }
            if match_pattern!(black, rev=true, $pattern) && match_long_pattern_for_black!($position, rev) {
                $(
                    apply_patch!(black, rev=true, $patch);
                )*
            }
        };
        ($color:ident,rev=$rev:expr,$pattern:literal,$($patch:literal),+) => {
            if match_pattern!($color, rev=$rev, $pattern) {
                $(
                    apply_patch!($color, rev=$rev, $patch);
                )*
            }
        };
    }

    // TODO: STRONG control hazard, needs optimization.

    // THREE

    process_pattern!(black, asymmetry, "!.OO...!", "!.OO3..!", "!.OO.3.!");
    process_pattern!(black, asymmetry, "!..OO..X", "!..OO3.X");
    process_pattern!(black, asymmetry, long-pattern, left, "..OO...O", "..OO3..O");
    process_pattern!(black, asymmetry, "!.O.O..!", "!.O3O..!", "!.O.O3.!");

    process_pattern!(white, asymmetry, ".OO...", ".OO3..", ".OO.3.");
    process_pattern!(white, asymmetry, "..OO..X", "..OO3.X");
    process_pattern!(white, asymmetry, ".O.O..", ".O3O..", ".O.O3.");

    // CLOSED-FOUR

    process_pattern!(black, symmetry, "!O.O.O!", "!OFO.O!", "!O.OFO!");
    process_pattern!(black, asymmetry, "!OO.O.!", "!OOFO.!", "!OO.OF!");
    process_pattern!(black, asymmetry, "!O.OO.!", "!O.OOF!");
    process_pattern!(black, asymmetry, "!OO..O!", "!OOF.O!", "!OO.FO!");

    process_pattern!(black, asymmetry, "XOO.O.!", "XOOFO.!", "XOO.O.F!");
    process_pattern!(black, asymmetry, "XOOO..!", "XOOOF.!", "XOOO.F!");
    process_pattern!(black, asymmetry, "!..OOO.X", "!..OOOFX");

    process_pattern!(white, symmetry, "O.O.O", "OFO.O", "O.OFO");
    process_pattern!(white, asymmetry, "OO.O.", "OOFO.", "OO.OF");
    process_pattern!(white, asymmetry, "O.OO.", "O.OOF");
    process_pattern!(white, asymmetry, "OO..O", "OOF.O", "OO.FO");

    process_pattern!(white, asymmetry, "XOO.O.", "XOOFO.", "XOO.OF");
    process_pattern!(white, asymmetry, "XOOO..", "XOOOF.", "XOOO.F");
    process_pattern!(white, asymmetry, "..OOO.X", "..OOOFX");

    // OPEN-FOUR

    process_pattern!(black, asymmetry, "!.OOO..!", "!.OOO4.!", "!.OOO.F!", "!COOO..!", "!.OOOC.!");
    process_pattern!(black, asymmetry, "!.OO.O.!", "!.OO4O.!", "!COO.O.!", "!.OOCO.!", "!.OO.OC!");

    process_pattern!(white, asymmetry, ".OOO..", ".OOO4.", ".OOO.F", "COOO..", ".OOOC.");
    process_pattern!(white, asymmetry, ".OO.O.", ".OO4O.", "COO.O.", ".OOCO.", ".OO.OC");

    // CLOSE-THREE

    process_pattern!(black, asymmetry, long-pattern, right, "O..OOO..", "O..OOO.C");
    process_pattern!(black, asymmetry, "X.OOO..!", "XCOOO..!", "X.OOO.C!");

    // FIVE

    process_pattern!(black, symmetry, "!OO.OO!", "!OO5OO!");
    process_pattern!(black, asymmetry, "!OOO.O!", "!OOO5O!");
    process_pattern!(black, asymmetry, "!OOOO.!", "!OOOO5!");

    process_pattern!(white, symmetry, "OO.OO", "OO5OO");
    process_pattern!(white, asymmetry, "OOO.O", "OOO5O");
    process_pattern!(white, asymmetry, "OOOO.", "OOOO5");

    // OVERLINE

    process_pattern!(black, asymmetry, "O.OOOO", "O6OOOO");
    process_pattern!(black, asymmetry, "OO.OOO", "OO6OOO");

    // WIN

    if b & 0b000_11111 == 0b000_11111 {
        acc.winner = Some(Color::Black);
        return;
    }
    if w & 0b000_11111 == 0b000_11111 {
        acc.winner = Some(Color::White);
    }

}

fn increase_closed_four(packed: u8) -> u8 {
    packed | (0b1000_0000 >> (packed >> 7))
}

const fn build_pattern_mask(source: &str, reversed: bool) -> u32 {
    parse_pattern_literal('O', source, reversed) as u32
        | (parse_pattern_literal('!', source, reversed) as u32) << 8
        | (parse_pattern_literal('.', source, reversed) as u32) << 16
        | (parse_pattern_literal('X', source, reversed) as u32) << 24
}

const fn build_pattern_result(source: &str, reversed: bool) -> u32 {
    parse_pattern_literal('O', source, reversed) as u32
        | (parse_pattern_literal('.', source, reversed) as u32) << 16
        | (parse_pattern_literal('X', source, reversed) as u32) << 24
}

const fn parse_pattern_literal(kind: char, source: &str, reversed: bool) -> u8 {
    let mut acc: u8 = 0;
    let mut idx: usize = 0;
    while idx < source.len() {
        let pos = if reversed { 7 - idx } else { idx };
        if source.as_bytes()[idx] as char == kind {
            acc |= 0b1 << pos;
        }

        idx += 1;
    }

    acc
}

const fn parse_patch_literal(source: &str, reversed: bool) -> (isize, u8) {
    let mut idx: isize = 0;
    while idx < source.len() as isize {
        let pos = if reversed { 7 - idx } else { idx };
        match source.as_bytes()[idx as usize] as char {
            '3' => return (pos, OPEN_THREE),
            'C' => return (pos, CLOSE_THREE),
            '4' => return (pos, OPEN_FOUR),
            'F' => return (pos, CLOSED_FOUR_SINGLE),
            '5' => return (pos, FIVE),
            '6' => return (pos, INV_THREE_OVERLINE),
            _ => {}
        }
        idx += 1;
    }

    unreachable!()
}
