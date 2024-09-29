use crate::notation::color::Color;
use crate::notation::pos;
use crate::pattern::{CLOSED_FOUR_SINGLE, CLOSE_THREE, FIVE, INV_THREE_OVERLINE, OPEN_FOUR, OPEN_THREE};
use crate::slice::Slice;
use crate::{max, min};

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct SlicePattern {
    pub black_patterns: [u8; 16], // 128-bits
    pub white_patterns: [u8; 16],
    pub five_in_a_row: Option<(u8, Color)>
}

impl SlicePattern {

    pub fn is_empty(&self) -> bool {
        u128::from_ne_bytes(self.black_patterns) == 0 && u128::from_ne_bytes(self.white_patterns) == 0
    }

}

pub const EMPTY_SLICE_PATTERN: SlicePattern = SlicePattern {
    black_patterns: [0; 16],
    white_patterns: [0; 16],
    five_in_a_row: None,
};

impl Slice {

    pub fn calculate_slice_pattern(&self) -> SlicePattern {
        // padding = 3
        let wall: u32 = !(!(u32::MAX << self.length as u32) << 3);
        let b: u32 = (self.black_stones as u32) << 3;
        let w: u32 = (self.white_stones as u32) << 3;
        let bw = b | wall;
        let ww = w | wall;
        let cold = !(bw | ww);

        let mut acc: SlicePattern = EMPTY_SLICE_PATTERN;
        for shift in 0 ..= self.length as usize + 1 { // length - 5 + 3 * 2
            let cold_frag = (cold >> shift) as u8;
            if !(b.count_ones() < 2 && w.count_ones() < 2) && cold != 0 {
                find_patterns(
                    &mut acc, shift, shift as isize - 3,
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

// big endian system is NOT supported
#[allow(clippy::too_many_arguments)]
#[inline]
fn find_patterns(
    acc: &mut SlicePattern,
    shift: usize, offset: isize,
    ob: u32, b: u8, w: u8, bw: u8, ww: u8, cold: u8
) {
    /*
    ## PATTERN-MATCH-LITERAL
    * O = self-color-hot
    * X = reversed-color-hot
    * ! = not self-color-hot
    * . = cold

    > EX: match black's closed-four = "!OOO..!"

    ## PATTERN-PATTERN-LITERAL
    * 3 = open-three
    * C = close-three
    * 4 = open-four
    * F = closed-four-single
    * 5 = five
    * 6 = overline

    > EX: match black's closed-four = "!OOO.F"
    */

    let b_u32_vector: u32 = b as u32 | (b as u32) << 8 | (cold as u32) << 16 | (ww as u32) << 24;
    let w_u32_vector: u32 = w as u32 | (w as u32) << 8 | (cold as u32) << 16 | (bw as u32) << 24;

    macro_rules! match_long_pattern_for_black {
        (left, rev) => (match_long_pattern_for_black!(right));
        (right, rev) => (match_long_pattern_for_black!(left));
        (left) => {(
            ob & 0b1 << offset + 1 == 0 // offset + 3 - 1 - 1
        )};
        (right) => {(
            ob & 0b1 << offset + 11 == 0 // offset + 3 + 8 + 1 - 1
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

    macro_rules! apply_single_patch {
        (black,rev=$rev:expr,$patch:literal) => (apply_single_patch!(acc.black_patterns,rev=$rev,$patch));
        (white,rev=$rev:expr,$patch:literal) => (apply_single_patch!(acc.white_patterns,rev=$rev,$patch));
        ($patterns_expr:expr,rev=$rev:expr,$patch:literal) => {{
             const POS_KIND_TUPLE: (u8, u8) = parse_patch_literal($patch, $rev);

            // branch removed at compile time
            if (POS_KIND_TUPLE.1 == CLOSED_FOUR_SINGLE) {
                let original = $patterns_expr[(offset + POS_KIND_TUPLE.0 as isize) as usize];
                $patterns_expr[(offset + POS_KIND_TUPLE.0 as isize) as usize] = increase_closed_four_single(original);
            } else {
                $patterns_expr[(offset + POS_KIND_TUPLE.0 as isize) as usize] |= POS_KIND_TUPLE.1;
            }
        }};
    }
    
    macro_rules! apply_multiple_patch {
        (black,rev=$rev:expr,$($patch:literal),+) => (apply_multiple_patch!(acc.black_patterns,rev=$rev, $($patch),+));
        (white,rev=$rev:expr,$($patch:literal),+) => (apply_multiple_patch!(acc.white_patterns,rev=$rev, $($patch),+));
        ($patterns_expr:expr,rev=$rev:expr,$($patch:literal),+) => {{
            const PATCH_MASK_LUT: SlicePatchMaskLUT = build_slice_patch_mask_lut([$($patch),*], $rev);

            let mut original = u128::from_ne_bytes($patterns_expr);

            let slice_patch_mask = PATCH_MASK_LUT.look_up_table[shift];
            // branches removed at compile time (NO branching)
            if PATCH_MASK_LUT.include_non_closed_four {
                original |= slice_patch_mask.patch_mask;
            }
            if PATCH_MASK_LUT.include_closed_four {
                original = increase_closed_four_multiple(original,
                    slice_patch_mask.closed_four_clear_mask,
                    slice_patch_mask.closed_four_mask
                );
            }

            $patterns_expr = original.to_ne_bytes();
        }};
    }

    macro_rules! apply_patch_matcher {
        ($color:ident,rev=$rev:expr,$a:literal) =>
            (apply_single_patch!($color,rev=$rev,$a));
        ($color:ident,rev=$rev:expr,$a:literal,$b:literal) =>
            (apply_patch_matcher!($color,rev=$rev,$a,$b,"",""));
        ($color:ident,rev=$rev:expr,$a:literal,$b:literal,$c:literal) =>
            (apply_patch_matcher!($color,rev=$rev,$a,$b,$c,""));
        ($color:ident,rev=$rev:expr,$a:literal,$b:literal,$c:literal,$d:literal) =>
            (apply_multiple_patch!($color,rev=$rev,$a,$b,$c,$d));
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
                apply_patch_matcher!(black, rev=false, $($patch),+);
            }
            if match_pattern!(black, rev=true, $pattern) && match_long_pattern_for_black!($position, rev) {
                apply_patch_matcher!(black, rev=true, $($patch),+);
            }
        };
        ($color:ident,rev=$rev:expr,$pattern:literal,$($patch:literal),+) => {
            if match_pattern!($color, rev=$rev, $pattern) {
                apply_patch_matcher!($color, rev=$rev, $($patch),+);
            }
        };
    }

    // TODO: STRONG control hazard, needs optimization.
    // try binary search for optimization?

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

    const FIVE_MASK: u8 = 0b_11111;

    if b & FIVE_MASK == FIVE_MASK {
        acc.five_in_a_row = Some((offset as u8, Color::Black));
    } else if w & FIVE_MASK == FIVE_MASK {
        acc.five_in_a_row = Some((offset as u8, Color::White));
    }

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

#[derive(Copy, Clone)]
struct SlicePatchMask {
    pub patch_mask: u128,
    pub closed_four_clear_mask: u128,
    pub closed_four_mask: u128,
}

const MASK_LUT_SIZE: usize = pos::U_BOARD_WIDTH + 1;

struct SlicePatchMaskLUT {
    pub look_up_table: [SlicePatchMask; MASK_LUT_SIZE],
    pub include_non_closed_four: bool,
    pub include_closed_four: bool
}

// big-endian not supported
const fn build_slice_patch_mask_lut(sources: [&str; 4], reversed: bool) -> SlicePatchMaskLUT {
    let original = unsafe {
        let mut patch_mask: [u8; 16] = [0; 16];
        let mut closed_four_clear_mask: [u8; 16] = [0; 16];
        let mut closed_four_mask: [u8; 16] = [0; 16];

        let mut idx: usize = 0;
        while idx < 4 {
            if sources[idx].len() > 1 {
                let (pos, kind) = parse_patch_literal(sources[idx], reversed);

                if kind == CLOSED_FOUR_SINGLE {
                    closed_four_clear_mask[pos as usize] = 0b1100_0000;
                    closed_four_mask[pos as usize] = CLOSED_FOUR_SINGLE;
                } else {
                    patch_mask[pos as usize] |= kind;
                }
            }
            idx += 1;
        }

        SlicePatchMask {
            patch_mask: u128::from_ne_bytes(patch_mask),
            closed_four_clear_mask: u128::from_ne_bytes(closed_four_clear_mask),
            closed_four_mask: u128::from_ne_bytes(closed_four_mask),
        }
    };

    let look_up_table = {
        let mut lut = [SlicePatchMask {
            patch_mask: 0, closed_four_clear_mask: 0, closed_four_mask: 0
        }; MASK_LUT_SIZE];

        let mut idx: isize = 0;
        while idx < MASK_LUT_SIZE as isize {
            let shl = min!(0, idx - 3).abs() * 8;
            let shr = max!(0, idx - 3) * 8;

            lut[idx as usize] = SlicePatchMask {
                patch_mask: (original.patch_mask << shr) >> shl,
                closed_four_clear_mask: (original.closed_four_clear_mask << shr) >> shl,
                closed_four_mask: (original.closed_four_mask << shr) >> shl,
            };

            idx += 1;
        }

        lut
    };

    SlicePatchMaskLUT {
        look_up_table,
        include_non_closed_four: original.patch_mask != 0,
        include_closed_four: original.closed_four_mask != 0,
    }
}

fn increase_closed_four_single(packed: u8) -> u8 {
    packed | (0b1000_0000 >> (packed >> 7))
}

// big-endian not supported
fn increase_closed_four_multiple(original: u128, clear_mask: u128, mask: u128) -> u128 {
    let mut copid: u128 = original;     // 0 0 0 | 1 0 0 | 1 1 0
    copid >>= 1;                        // 0 0 0 | 0 1 0 | 0 1 1
    copid |= mask;                      // 1 0 0 | 1 1 0 | 1 1 1
    copid &= clear_mask;                // 1 0 0 | 1 1 0 | 1 1 0
    original | copid                    // empty, four*1, four*2
}

const fn parse_patch_literal(source: &str, reversed: bool) -> (u8, u8) {
    let mut idx: u8 = 0;
    while idx < source.len() as u8 {
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
