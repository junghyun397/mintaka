use crate::notation::color::Color;
use crate::pattern::{CLOSED_FOUR_SINGLE, CLOSE_THREE, FIVE, OPEN_FOUR, OPEN_THREE, OVERLINE};
use crate::slice::Slice;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SlicePattern(pub [u8; 16]);

impl SlicePattern {

    pub const EMPTY: Self = Self([0; 16]);

}

impl Slice {

    pub fn calculate_slice_pattern_alt<const C: Color>(&self) -> SlicePattern {
        // padding = 3
        let block: u32 = !(!(u32::MAX << self.length as u32) << 3);
        let extended_p: u32 = (self.stones::<C>() as u32) << 3;
        let extended_q: u32 = (self.stones_reversed::<C>() as u32) << 3;
        let qb = extended_q | block;

        let mut acc: SlicePattern = SlicePattern::EMPTY;
        for shift in 0 ..= self.length as usize + 1 { // length - 5 + 3 * 2
            let p = (extended_p >> shift) as u16 & 0x00FF;

            if p.count_ones() > 1 {
                find_patterns::<C>(
                    &mut acc, shift, shift as isize - 3,
                    p | (((qb >> shift) as u16 & 0x00FF) << 8),
                    extended_p
                );
            }
        }

        acc
    }

}

#[inline]
fn find_patterns<const C: Color>(
    acc: &mut SlicePattern,
    shift: usize, offset: isize,
    vector: u16,
    raw: u32,
) {
    #[cold]
    #[inline(always)]
    fn extended_match_for_black(direction: ExtendedMatch, b_raw: u32, offset: isize) -> bool {
        match direction {
            ExtendedMatch::Left => b_raw & (0b1 << offset + 2) == 0,
            ExtendedMatch::Right => b_raw & (0b1 << offset + 11) == 0,
            _ => unreachable!()
        }
    }

    let patch_idx = match C {
        Color::Black => SLICE_PATTERN_LUT.vector.black[vector as usize],
        Color::White => SLICE_PATTERN_LUT.vector.white[vector as usize],
    };

    if patch_idx != 0 {
        let slice_patch_data = SLICE_PATTERN_LUT.patch.black[patch_idx as usize];

        if C == Color::Black
            && slice_patch_data.extended_match != None
            && extended_match_for_black(slice_patch_data.extended_match.unwrap(), raw, offset)
        {
            return;
        }

        let shl = 0.min(shift as isize - 3).abs() * 8;
        let shr = 0.max(shift as isize - 3) * 8;

        let mut original = u128::from_ne_bytes(acc.0);
        if slice_patch_data.contains_patch_mask {
            original |= ((slice_patch_data.patch_mask << shr) >> shl) as u128;
        }
        if slice_patch_data.contains_closed_four {
            original = increase_closed_four_multiple(
                original,
                ((slice_patch_data.closed_four_clear_mask << shr) >> shl) as u128,
                ((slice_patch_data.closed_four_mask << shr) >> shl) as u128
            )
        }

        acc.0 = original.to_ne_bytes();
    }
}

fn calculate_five_in_a_rows(mut stones: u16) -> bool {
    stones &= stones >> 1;  // 1 1 1 1 1 0 & 0 1 1 1 1 1
    stones &= stones >> 3;  // 0 1 1 1 1 0 & 0 0 0 0 1 1 ...
    stones != 0             // 0 0 0 0 1 0 ...
}

fn increase_closed_four_multiple(original: u128, clear_mask: u128, mask: u128) -> u128 {
    let mut copied: u128 = original;     // 0 0 0 | 1 0 0 | 1 1 0
    copied >>= 1;                        // 0 0 0 | 0 1 0 | 0 1 1
    copied |= mask;                      // 1 0 0 | 1 1 0 | 1 1 1
    copied &= clear_mask;                // 1 0 0 | 1 1 0 | 1 1 0
    original | copied                    // empty   slot 1  slot 2
}

struct SlicePatternLut {
    vector: VectorMatchLut,
    patch: PatchLut,
}

// 32 KiB
struct VectorMatchLut {
    black:      [u8; u16::MAX as usize],
    white:      [u8; u16::MAX as usize],
}

struct PatchLut {
    black: [SlicePatchData; u8::MAX as usize],
    white: [SlicePatchData; u8::MAX as usize],
}

const SLICE_PATTERN_LUT: SlicePatternLut = build_slice_pattern_lut();

const fn build_slice_pattern_lut() -> SlicePatternLut {
    let mut slice_pattern_lut = {
        let initial_patch_data = SlicePatchData {
            patch_mask: 0,
            closed_four_clear_mask: 0,
            closed_four_mask: 0,
            contains_patch_mask: false,
            contains_closed_four: false,
            extended_match: None
        };

        SlicePatternLut {
            vector: VectorMatchLut {
                black: [0; u16::MAX as usize],
                white: [0; u16::MAX as usize],
            },
            patch: PatchLut {
                black: [initial_patch_data; u8::MAX as usize],
                white: [initial_patch_data; u8::MAX as usize],
            },
        }
    };

    let mut patch_black_idx: usize = 1;
    let mut patch_white_idx: usize = 1;

    macro_rules! sources {
        ($a:expr) => ([$a, "", "", ""]);
        ($a:expr,$b:expr) => ([$a, $b, "", ""]);
        ($a:expr,$b:expr,$c:expr) => ([$a, $b, $c, ""]);
        ($a:expr,$b:expr,$c:expr,$d:expr) => ([$a, $b, $c, $d]);
    }

    macro_rules! flash_pattern {
        ($vector_expr:expr, $idx_expr:expr, $extended_match:expr, $rev:expr, $sources:expr) => {{
            let overlap = slice_pattern_lut.vector.black[0] != 0;

            let target_idx = if overlap {
                $vector_expr[0] as usize
            } else {
                $idx_expr
            };

            $vector_expr[target_idx] = build_slice_patch_data($extended_match, $rev, $sources);

            if !overlap {
                $idx_expr += 1;
            }

            target_idx
        }};
    }

    // slice_pattern_lut.pattern.

    macro_rules! embed_pattern {
        (black,asymmetry,long-pattern,$position:ident,$pattern:literal,$($patch:literal),+) => {
        };
        ($color:ident,symmetry,$pattern:literal,$($patch:literal),+) => {
            embed_pattern!($color, rev=false, $pattern, $($patch),+);
        };
        ($color:ident,asymmetry,$pattern:literal,$($patch:literal),+) => {
            embed_pattern!($color, rev=false, $pattern, $($patch),+);
            embed_pattern!($color, rev=true, $pattern, $($patch),+)
        };
        ($color:ident,rev=$rev:expr,$pattern:literal,$($patch:literal),+) => {
            // let patch_idx = flash_pattern!(slice_pattern_lut.vector.black, patch_black_idx, ExtendedMatch::None, $rev, sources!($($patch),+));
            // slice_pattern_lut.vector.black[0] = patch_idx;
        };
    }

    // black-open-three

    embed_pattern!(black, asymmetry, "!.OO...!", "!.OO3..!", "!.OO.3.!");
    embed_pattern!(black, asymmetry, "X..OO..!", "X.3OO..!");
    embed_pattern!(black, asymmetry, "!.O.O..!", "!.O3O..!", "!.O.O3.!");
    embed_pattern!(black, symmetry, "!.O..O.!", "!.O3.O.!", "!.O.3O.!");
    embed_pattern!(black, asymmetry, long-pattern, left, "..OO...O", "..OO3..O"); // [!]..OO...O

    // white-open-three

    embed_pattern!(white, asymmetry, ".OO...", ".OO.3.");
    embed_pattern!(white, asymmetry, "!.OO...", "!.OO3..");
    embed_pattern!(white, asymmetry, "X..OO..", "X.3OO..");
    embed_pattern!(white, asymmetry, ".O.O..", ".O.O3.");
    embed_pattern!(white, asymmetry, "!.O.O..", "!.O3O..");
    embed_pattern!(white, asymmetry, "!.O..O.", "!.O3.O.");
    embed_pattern!(white, asymmetry, "!O.O..O.", "!O.O3.O.");

    // black-closed-four

    embed_pattern!(black, symmetry, "!O.O.O!", "!OFO.O!", "!O.OFO!");
    embed_pattern!(black, asymmetry, "!OO.O.!", "!OO.OF!");
    embed_pattern!(black, asymmetry, "!O.OO.!", "!O.OOF!");
    embed_pattern!(black, asymmetry, "!OO..O!", "!OOF.O!", "!OO.FO!");

    embed_pattern!(black, asymmetry, "XOOO..!", "XOOOF.!", "XOOO.F!");
    embed_pattern!(black, asymmetry, "XOO.O.!", "XOOFO.!");
    embed_pattern!(black, asymmetry, "XO.OO.!", "XOFOO.!");
    embed_pattern!(black, asymmetry, "X.OOO.!", "XFOOO.!");
    embed_pattern!(black, asymmetry, "X.OOO..!", "X.OOO.C!");

    embed_pattern!(black, asymmetry, "O.O.OO.!", "O.OFOO.!");
    embed_pattern!(black, asymmetry, "O.OO.O.!", "O.OOFO.!");
    embed_pattern!(black, asymmetry, long-pattern, left, "..OOO..O", "..OOOF.O", "C.OOO..O"); // [!]..OOO..O

    // white-closed-four

    embed_pattern!(white, symmetry, "!O.O.O!", "!OFO.O!", "!O.OFO!");
    embed_pattern!(white, asymmetry, "OOO..!", "OOO.F!");

    embed_pattern!(white, symmetry, "OO..OO", "OOF.OO", "OO.FOO");
    embed_pattern!(white, asymmetry, "OO..O!", "OOF.O!", "OO.FO!");
    embed_pattern!(white, asymmetry, "OO.O.O!", "OO.OFO!");

    embed_pattern!(white, asymmetry, "OO.O.!", "OO.OF!");
    embed_pattern!(white, asymmetry, "O.OO.!", "O.OOF!");

    embed_pattern!(white, asymmetry, "XOOO..!", "XOOOF.!");
    embed_pattern!(white, asymmetry, "XOO.O.", "XOOFO.");
    embed_pattern!(white, asymmetry, "XO.OO.", "XOFOO.");
    embed_pattern!(white, asymmetry, "X.OOO.", "XFOOO.");
    embed_pattern!(white, asymmetry, "X.OOO..", "X.OOO.C");

    // black open-four

    embed_pattern!(black, asymmetry, "!.OOO..!", "!.OOO4.!", "!.OOO.F!", "!COOO..!", "!.OOOC.!");
    embed_pattern!(black, asymmetry, "!.OO.O.!", "!.OO4O.!", "!COO.O.!", "!.OOCO.!", "!.OO.OC!");

    // white-open-four

    embed_pattern!(white, asymmetry, ".OOO..", ".OOO4.", "COOO..", ".OOOC.");
    embed_pattern!(white, asymmetry, ".OO.O.", ".OO4O.", "COO.O.", ".OOCO.", ".OO.OC");

    // black-five

    embed_pattern!(black, symmetry, "!OO.OO!", "!OO5OO!");
    embed_pattern!(black, asymmetry, "!OOO.O!", "!OOO5O!");
    embed_pattern!(black, asymmetry, "!OOOO.!", "!OOOO5!");

    // white-five

    embed_pattern!(white, symmetry, "OO.OO", "OO5OO");
    embed_pattern!(white, asymmetry, "OOO.O", "OOO5O");
    embed_pattern!(white, asymmetry, "OOOO.", "OOOO5");

    // black-overline

    embed_pattern!(black, asymmetry, "O.OOOO", "O6OOOO");
    embed_pattern!(black, asymmetry, "OO.OOO", "OO6OOO");

    slice_pattern_lut
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ExtendedMatch {
    Left,
    Right
}

#[derive(Copy, Clone)]
struct SlicePatchData {
    pub patch_mask: u64,
    pub closed_four_clear_mask: u64,
    pub closed_four_mask: u64,
    pub contains_patch_mask: bool,
    pub contains_closed_four: bool,
    pub extended_match: Option<ExtendedMatch>,
}

const fn build_slice_patch_data(extended_match: Option<ExtendedMatch>, reversed: bool, sources: [&str; 4]) -> SlicePatchData {
    let mut patch_mask: [u8; 8] = [0; 8];
    let mut closed_four_clear_mask: [u8; 8] = [0; 8];
    let mut closed_four_mask: [u8; 8] = [0; 8];

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

    let patch_mask = u64::from_ne_bytes(patch_mask);
    let closed_four_clear_mask = u64::from_ne_bytes(closed_four_clear_mask);
    let closed_four_mask = u64::from_ne_bytes(closed_four_mask);

    let extended_match = if reversed {
        match extended_match {
            Some(ExtendedMatch::Left) => Some(ExtendedMatch::Right),
            Some(ExtendedMatch::Right) => Some(ExtendedMatch::Left),
            _ => None,
        }
    } else {
        extended_match
    };

    SlicePatchData {
        contains_patch_mask: patch_mask != 0,
        contains_closed_four: closed_four_mask != 0,
        patch_mask,
        closed_four_clear_mask,
        closed_four_mask,
        extended_match,
    }
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
            '6' => return (pos, OVERLINE),
            _ => {}
        }
        idx += 1;
    }

    unreachable!()
}
