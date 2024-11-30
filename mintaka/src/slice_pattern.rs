use crate::notation::color::Color;
use crate::notation::pos;
use crate::pattern::{CLOSED_FOUR_SINGLE, CLOSE_THREE, FIVE, OPEN_FOUR, OPEN_THREE, OVERLINE};
use crate::slice::Slice;
use crate::{max, min};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct SlicePattern {
    pub patterns: [u8; 16]
}

impl SlicePattern {

    pub const EMPTY: Self = Self { patterns: [0; 16] };

    pub fn is_empty(&self) -> bool {
        u128::from_ne_bytes(self.patterns) == 0
    }

}

impl Slice {

    pub fn calculate_slice_pattern<const C: Color>(&self) -> SlicePattern {
        // padding = 3
        let block: u32 = !(!(u32::MAX << self.length as u32) << 3);
        let extended_p: u32 = (self.stones::<C>() as u32) << 3;
        let extended_q: u32 = (self.stones_reversed::<C>() as u32) << 3;
        let qb = extended_q | block;
        let cold = !(extended_q | extended_p | block);

        let mut acc: SlicePattern = SlicePattern::EMPTY;
        for shift in 0 ..= self.length as usize + 1 { // length - 5 + 3 * 2
            let p = (extended_p >> shift) as u8;
            let q = (extended_q >> shift) as u8;
            let c = (cold >> shift) as u8;

            if p.count_ones() > 1 && c != u8::MAX && (
                p & !(q << 1) & !(q >> 1) != 0
            ) {
                find_patterns::<C>(
                    &mut acc, shift, shift as isize - 3,
                    (extended_p >> shift) as u8, (cold >> shift) as u8, (qb >> shift) as u8,
                    extended_p
                );
            }
        }

        acc
    }

}

// big endian system is NOT supported
#[allow(clippy::too_many_arguments)]
fn find_patterns<const C: Color>(
    acc: &mut SlicePattern,
    shift: usize, offset: isize,
    p: u8, cold: u8, qb: u8,
    raw: u32
) {
    /*
    ## PATTERN-MATCH-LITERAL
    * O = self-color-hot
    * X = reversed-color-hot
    * ! = not self-color-hot
    * . = cold

    ## PATTERN-APPLY-LITERAL
    * 3 = open-three
    * C = close-three
    * 4 = open-four
    * F = closed-four-single
    * 5 = five
    * 6 = overline
    */

    let vector: u32 = p as u32 | (p as u32) << 8 | (cold as u32) << 16 | (qb as u32) << 24;

    macro_rules! match_long_pattern_for_black {
        (left, rev) => (match_long_pattern_for_black!(right));
        (right, rev) => (match_long_pattern_for_black!(left));
        (left) => {(
            raw & (0b1 << offset + 2) == 0
        )};
        (right) => {(
            raw & (0b1 << offset + 11) == 0
        )};
    }

    macro_rules! match_pattern {
        ($color:ident,$pattern:literal) => (match_pattern!($color, rev=false, $pattern));
        (black,rev=$rev:expr,$pattern:literal) => {C == Color::Black && {
            const MASK: u32 = build_pattern_mask($pattern, $rev);
            const RESULT: u32 = build_pattern_result($pattern, $rev);

            vector & MASK == RESULT
        }};
        (white,rev=$rev:expr,$pattern:literal) => {C == Color::White && {
            const MASK: u32 = build_pattern_mask($pattern, $rev);
            const RESULT: u32 = build_pattern_result($pattern, $rev);

            vector & MASK == RESULT
        }};
    }

    macro_rules! apply_single_patch {
        (black,rev=$rev:expr,$patch:literal) => (apply_single_patch!(rev=$rev,$patch));
        (white,rev=$rev:expr,$patch:literal) => (apply_single_patch!(rev=$rev,$patch));
        (rev=$rev:expr,$patch:literal) => {{
            const POS_KIND_TUPLE: (isize, u8) = parse_patch_literal($patch, $rev);

            if (POS_KIND_TUPLE.1 == CLOSED_FOUR_SINGLE) {
                let original = acc.patterns[(offset + POS_KIND_TUPLE.0) as usize];
                acc.patterns[(offset + POS_KIND_TUPLE.0) as usize] = increase_closed_four_single(original);
            } else {
                acc.patterns[(offset + POS_KIND_TUPLE.0) as usize] |= POS_KIND_TUPLE.1;
            }
        }};
    }
    
    macro_rules! apply_multiple_patch {
        (black,rev=$rev:expr,$($patch:literal),+) => (apply_multiple_patch!(rev=$rev, $($patch),+));
        (white,rev=$rev:expr,$($patch:literal),+) => (apply_multiple_patch!(rev=$rev, $($patch),+));
        (rev=$rev:expr,$($patch:literal),+) => {{
            const PATCH_MASK_LUT: SlicePatchMaskLUT = build_slice_patch_mask_lut([$($patch),*], $rev);

            let mut original = u128::from_ne_bytes(acc.patterns);

            let slice_patch_mask = PATCH_MASK_LUT.look_up_table[shift];
            if PATCH_MASK_LUT.contains_non_closed_four {
                original |= slice_patch_mask.patch_mask;
            }
            if PATCH_MASK_LUT.contains_closed_four {
                original = increase_closed_four_multiple(
                    original,
                    slice_patch_mask.closed_four_clear_mask,
                    slice_patch_mask.closed_four_mask
                );
            }

            acc.patterns = original.to_ne_bytes();
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

    // black-open-three

    process_pattern!(black, asymmetry, "!.OO...!", "!.OO3..!", "!.OO.3.!");
    process_pattern!(black, asymmetry, "X..OO..!", "X.3OO..!");
    process_pattern!(black, asymmetry, "!.O.O..!", "!.O3O..!", "!.O.O3.!");
    process_pattern!(black, symmetry, "!.O..O.!", "!.O3.O.!", "!.O.3O.!");
    process_pattern!(black, asymmetry, long-pattern, left, "..OO...O", "..OO3..O"); // [!]..OO...O

    // white-open-three

    process_pattern!(white, asymmetry, ".OO...", ".OO.3.");
    process_pattern!(white, asymmetry, "!.OO...", "!.OO3..");
    process_pattern!(white, asymmetry, "X..OO..", "X.3OO..");
    process_pattern!(white, asymmetry, ".O.O..!!", ".O.O3.!!");
    process_pattern!(white, asymmetry, ".O.O..O!", ".O.O3.O!");
    process_pattern!(white, asymmetry, "!.O.O..", "!.O3O..");
    process_pattern!(white, asymmetry, "!.O..O.", "!.O3.O.");
    process_pattern!(white, asymmetry, "!O.O..O.", "!O.O3.O.");

    // black-closed-four

    process_pattern!(black, symmetry, "!O.O.O!", "!OFO.O!", "!O.OFO!");
    process_pattern!(black, asymmetry, "!OO.O.!", "!OO.OF!");
    process_pattern!(black, asymmetry, "!O.OO.!", "!O.OOF!");
    process_pattern!(black, asymmetry, "!OO..O!", "!OOF.O!", "!OO.FO!");

    process_pattern!(black, asymmetry, "XOOO..!", "XOOOF.!", "XOOO.F!");
    process_pattern!(black, asymmetry, "XOO.O.!", "XOOFO.!");
    process_pattern!(black, asymmetry, "XO.OO.!", "XOFOO.!");
    process_pattern!(black, asymmetry, "X.OOO.!", "XFOOO.!");
    process_pattern!(black, asymmetry, "X.OOO..!", "X.OOO.C!");

    process_pattern!(black, asymmetry, "O.O.OO.!", "O.OFOO.!");
    process_pattern!(black, asymmetry, "O.OO.O.!", "O.OOFO.!");
    process_pattern!(black, asymmetry, long-pattern, left, "..OOO..O", "..OOOF.O", "C.OOO..O"); // [!]..OOO..O

    // white-closed-four

    process_pattern!(white, symmetry, "!O.O.O!", "!OFO.O!", "!O.OFO!");
    process_pattern!(white, asymmetry, "OOO..!", "OOO.F!");

    process_pattern!(white, symmetry, "OO..OO", "OOF.OO", "OO.FOO");
    process_pattern!(white, asymmetry, "OO..O!", "OOF.O!", "OO.FO!");
    process_pattern!(white, asymmetry, "OO.O.O!", "OO.OFO!");

    process_pattern!(white, asymmetry, "OO.O.!", "OO.OF!");
    process_pattern!(white, asymmetry, "O.OO.!", "O.OOF!");

    process_pattern!(white, asymmetry, "XOOO..!", "XOOOF.!");
    process_pattern!(white, asymmetry, "XOO.O.", "XOOFO.");
    process_pattern!(white, asymmetry, "XO.OO.", "XOFOO.");
    process_pattern!(white, asymmetry, "X.OOO.", "XFOOO.");
    process_pattern!(white, asymmetry, "X.OOO..", "X.OOO.C");

    // black open-four

    process_pattern!(black, asymmetry, "!.OOO..!", "!.OOO4.!", "!.OOO.F!", "!COOO..!", "!.OOOC.!");
    process_pattern!(black, asymmetry, "!.OO.O.!", "!.OO4O.!", "!COO.O.!", "!.OOCO.!", "!.OO.OC!");

    // white-open-four

    process_pattern!(white, asymmetry, ".OOO..", ".OOO4.", "COOO..", ".OOOC.");
    process_pattern!(white, asymmetry, ".OO.O.", ".OO4O.", "COO.O.", ".OOCO.", ".OO.OC");

    // black-five

    process_pattern!(black, symmetry, "!OO.OO!", "!OO5OO!");
    process_pattern!(black, asymmetry, "!OOO.O!", "!OOO5O!");
    process_pattern!(black, asymmetry, "!OOOO.!", "!OOOO5!");

    // white-five

    process_pattern!(white, symmetry, "OO.OO", "OO5OO");
    process_pattern!(white, asymmetry, "OOO.O", "OOO5O");
    process_pattern!(white, asymmetry, "OOOO.", "OOOO5");

    // black-overline

    process_pattern!(black, asymmetry, "O.OOOO", "O6OOOO");
    process_pattern!(black, asymmetry, "OO.OOO", "OO6OOO");
}

fn calculate_four_in_a_rows(mut stones: u16) -> u16 {
    stones &= stones >> 1;
    stones &= stones >> 2;
    stones
}

fn calculate_five_in_a_rows(mut stones: u16) -> u16 {
    stones &= stones >> 1;  // 1 1 1 1 1 0 & 0 1 1 1 1 1
    stones &= stones >> 3;  // 0 1 1 1 1 0 & 0 0 0 0 1 1 ...
    stones                  // 0 0 0 0 1 0 ...
}

fn calculate_six_in_a_rows(mut stones: u16) -> u16 {
    stones &= stones >> 1;
    stones &= stones >> 1;
    stones &= stones >> 3;
    stones
}

pub fn contains_five_in_a_row(stones: u16) -> bool {
    calculate_five_in_a_rows(stones) != 0
}

pub fn contains_overline(stones: u16) -> bool {
    calculate_six_in_a_rows(stones) != 0
}

fn increase_closed_four_single(packed: u8) -> u8 {
    packed | (0b1000_0000 >> (packed >> 7))
}

fn increase_closed_four_multiple(original: u128, clear_mask: u128, mask: u128) -> u128 {
    let mut copied: u128 = original;     // 0 0 0 | 1 0 0 | 1 1 0
    copied >>= 1;                        // 0 0 0 | 0 1 0 | 0 1 1
    copied |= mask;                      // 1 0 0 | 1 1 0 | 1 1 1
    copied &= clear_mask;                // 1 0 0 | 1 1 0 | 1 1 0
    original | copied                    // empty   slot 1  slot 2
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
    pub contains_non_closed_four: bool,
    pub contains_closed_four: bool
}

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
        contains_non_closed_four: original.patch_mask != 0,
        contains_closed_four: original.closed_four_mask != 0,
    }
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
            '6' => return (pos, OVERLINE),
            _ => {}
        }
        idx += 1;
    }

    unreachable!()
}
