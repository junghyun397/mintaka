use crate::notation::color::Color;
use crate::pattern::{CLOSED_FOUR_SINGLE, CLOSE_THREE, FIVE, INV_THREE_OVERLINE, OPEN_FOUR, OPEN_THREE};
use crate::pop_count_less_then_two;
use crate::slice::Slice;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct SlicePattern {
    pub black_patch: [u8; 16], // 128-bits
    pub white_patch: [u8; 16],
    pub five_in_a_row: Option<(u8, Color)>
}

pub const EMPTY_SLICE_PATCH: SlicePattern = SlicePattern {
    black_patch: [0; 16],
    white_patch: [0; 16],
    five_in_a_row: None,
};

impl Slice {

    pub fn calculate_slice_patch(&self) -> SlicePattern {
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
#[inline]
fn find_patterns(
    acc: &mut SlicePattern,
    offset: isize,
    ob: u32, b: u8, w: u8, bw: u8, ww: u8, cold: u8
) {
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
        (black,rev=$rev:expr,$patch:literal) => {{
             const POS_KIND_TUPLE: (isize, u8) = parse_patch_literal($patch, $rev).unwrap();

            // branch removed at compile time
            if (POS_KIND_TUPLE.1 == CLOSED_FOUR_SINGLE) {
                let original = acc.black_patch[(offset + POS_KIND_TUPLE.0) as usize];
                acc.black_patch[(offset + POS_KIND_TUPLE.0) as usize] = increase_closed_four_single(original);
            } else {
                acc.black_patch[(offset + POS_KIND_TUPLE.0) as usize] |= POS_KIND_TUPLE.1;
            }
        }};
        (white,rev=$rev:expr,$patch:literal) => {{
            const POS_KIND_TUPLE: (isize, u8) = parse_patch_literal($patch, $rev).unwrap();

            // branch removed at compile time
            if (POS_KIND_TUPLE.1 == CLOSED_FOUR_SINGLE) {
                let original = acc.white_patch[(offset + POS_KIND_TUPLE.0) as usize];
                acc.white_patch[(offset + POS_KIND_TUPLE.0) as usize] = increase_closed_four_single(original);
            } else {
                acc.white_patch[(offset + POS_KIND_TUPLE.0) as usize] |= POS_KIND_TUPLE.1;
            }
        }};
    }

    macro_rules! apply_multiple_patch {
        (black,rev=$rev:expr,$($patch:literal),+) => {{
            const PATCH_MASK: SlicePatchMask = build_slice_patch([$($patch),*], $rev);

            let mut original: u128 = unsafe { std::mem::transmute(acc.black_patch) };

            let shr = std::cmp::min(0, offset).abs();
            let shl = std::cmp::max(0, offset);

            original |= (PATCH_MASK.patch_mask >> shr) << shl;
            // branch removed at compile time
            if PATCH_MASK.include_closed_four {
                original = increase_closed_four_multiple(original,
                    (PATCH_MASK.closed_four_clear_mask >> shr) << shl,
                    (PATCH_MASK.closed_four_mask >> shr) << shl
                );
            }

            acc.black_patch = unsafe { std::mem::transmute(original) };
            println!("{:?}", acc.black_patch);
        }};
        (white,rev=$rev:expr,$($patch:literal),+) => {{
            const PATCH_MASK: SlicePatchMask = build_slice_patch([$($patch),*], $rev);

            let mut original: u128 = unsafe { std::mem::transmute(acc.white_patch) };

            original |= PATCH_MASK.patch_mask;
            // branch removed at compile time
            if PATCH_MASK.include_closed_four {
                original = increase_closed_four_multiple(original,
                    PATCH_MASK.closed_four_clear_mask,
                    PATCH_MASK.closed_four_mask
                );
            }

            acc.white_patch = unsafe { std::mem::transmute(original) };
        }}
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
        return
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

struct SlicePatchMask {
    pub patch_mask: u128,
    pub closed_four_clear_mask: u128,
    pub closed_four_mask: u128,
    pub include_closed_four: bool,
}

// big-endian not supported
const fn build_slice_patch(sources: [&str; 4], reversed: bool) -> SlicePatchMask {
    const SHIFT_N: u32 = 128 - 8;
    const FOUR_CLEAR_MASK: u128 = 1100_0000 << SHIFT_N;
    const CLOSED_FOUR_SINGLE_MASK: u128 = (CLOSED_FOUR_SINGLE as u128) << SHIFT_N;

    let mut patch_mask: u128 = 0;
    let mut closed_four_clear_mask: u128 = 0;
    let mut closed_four_mask: u128 = 0;

    let mut idx: usize = 0;
    while idx < 4 {
        if let Some((pos, kind)) = parse_patch_literal(sources[idx], reversed) {
            if kind == CLOSED_FOUR_SINGLE {
                closed_four_clear_mask |= FOUR_CLEAR_MASK >> pos * 8;
                closed_four_mask |= CLOSED_FOUR_SINGLE_MASK >> pos * 8;
            } else {
                patch_mask |= ((kind as u128) << SHIFT_N) >> pos * 8;
            }
        }

        idx += 1;
    }

    SlicePatchMask {
        patch_mask,
        closed_four_clear_mask,
        closed_four_mask,
        include_closed_four: closed_four_mask != 0,
    }
}

fn increase_closed_four_single(packed: u8) -> u8 {
    packed | (0b1000_0000 >> (packed >> 7))
}

// big-endian not supported
fn increase_closed_four_multiple(original: u128, clear_mask: u128, mask: u128) -> u128 {
    let mut masked: u128 = original & clear_mask;   // 0 0 0 | 1 0 0 | 1 1 0
    masked <<= 1;                                   // 0 0 0 | 0 1 0 | 0 1 1
    masked |= mask;                                 // 1 0 0 | 1 1 0 | 1 1 1
    masked &= clear_mask;                           // 1 0 0 | 1 1 0 | 1 1 0
    original | masked                               // empty, four*1, four*2
}

const fn parse_patch_literal(source: &str, reversed: bool) -> Option<(isize, u8)> {
    let mut idx: isize = 0;
    while idx < source.len() as isize {
        let pos = if reversed { 7 - idx } else { idx };
        match source.as_bytes()[idx as usize] as char {
            '3' => return Some((pos, OPEN_THREE)),
            'C' => return Some((pos, CLOSE_THREE)),
            '4' => return Some((pos, OPEN_FOUR)),
            'F' => return Some((pos, CLOSED_FOUR_SINGLE)),
            '5' => return Some((pos, FIVE)),
            '6' => return Some((pos, INV_THREE_OVERLINE)),
            _ => {}
        }
        idx += 1;
    }

    None
}
