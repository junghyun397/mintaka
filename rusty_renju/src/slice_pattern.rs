use crate::notation::color::Color;
use crate::notation::rule::RuleKind;
use crate::slice::Slice;
use crate::slice_pattern::ExtendedMatch::{Left, Right};
use crate::{assert_struct_sizes, const_for, pattern};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct SlicePattern {
    pub patterns: u128
}

impl Default for SlicePattern {

    fn default() -> Self {
        Self::EMPTY
    }

}

impl SlicePattern {

    pub const EMPTY: Self = Self { patterns: 0 };

    pub fn is_empty(&self) -> bool {
        self.patterns == 0
    }

}

impl Slice {

    pub fn calculate_slice_pattern<const R: RuleKind, const C: Color>(&self) -> SlicePattern {
        // padding = 3
        let block: usize = !(!(usize::MAX << self.length as u32) << 3);
        let extended_p: usize = (self.stones::<C>() as usize) << 3;
        let extended_q: usize = ((self.stones_reversed_color::<C>() as usize) << 3) | block;

        let mut acc: SlicePattern = SlicePattern::EMPTY;
        for shift in 0 .. self.length as isize - 1 {
            let p = (extended_p >> shift) & 0x00FF;
            let q = (extended_q >> shift) & 0x00FF;

            // little-endian
            let shift_offset = (shift - 3) * 8;
            let shl = shift_offset.min(0).abs();
            let shr = shift_offset.max(0);

            lookup_patterns::<R, C>(
                &mut acc,
                shift,
                p | (q << 8),
                extended_p,
                shl,
                shr
            );
        }

        acc
    }

}

#[inline(always)]
fn lookup_patterns<const R: RuleKind, const C: Color>(
    acc: &mut SlicePattern,
    shift: isize,
    vector: usize,
    raw: usize,
    shl: isize,
    shr: isize
) {
    #[cold]
    fn extended_match_for_black(direction: ExtendedMatch, b_raw: usize, shift: isize) -> bool {
        match direction {
            Left => b_raw & (0b1 << (shift as usize).saturating_sub(1)) == 0,
            Right => b_raw & (0b1 << (shift + 8)) == 0
        }
    }

    let patch_pointer = match (R, C) {
        (RuleKind::Renju, Color::Black) => SLICE_PATTERN_LUT.vector.black[vector],
        _ => SLICE_PATTERN_LUT.vector.white[vector],
    };

    if patch_pointer != 0 {
        let slice_patch_data = unsafe { match C {
            Color::Black => SLICE_PATTERN_LUT.patch.black.get_unchecked(patch_pointer as usize),
            Color::White => SLICE_PATTERN_LUT.patch.white.get_unchecked(patch_pointer as usize),
        } };

        if R == RuleKind::Renju && C == Color::Black
            && slice_patch_data.extended_match.is_some_and(|extended_match|
            !extended_match_for_black(extended_match, raw, shift)
        ) {
            return;
        }

        if slice_patch_data.patch_mask != 0 {
            acc.patterns |= ((slice_patch_data.patch_mask as u128) << shr) >> shl;
        }

        if slice_patch_data.closed_four_mask != 0 {
            acc.patterns |= increase_closed_four(
                acc.patterns,
                ((slice_patch_data.closed_four_clear_mask as u128) << shr) >> shl,
                ((slice_patch_data.closed_four_mask as u128) << shr) >> shl
            );
        }
    }
}

#[inline(always)]
fn increase_closed_four(mut copied: u128, clear_mask: u128, mask: u128) -> u128 {
    copied >>= 1;                        // 0 0 0 | 0 1 0 | 0 1 1
    copied |= mask;                      // 1 0 0 | 1 1 0 | 1 1 1
    copied &= clear_mask;                // 1 0 0 | 1 1 0 | 1 1 0
    copied                               // 0 0 0 | 1 0 0 | 1 1 0
}

fn calculate_five_in_a_rows(stones: u16) -> u16 {
    stones
        & (stones >> 1)
        & (stones >> 2)
        & (stones >> 3)
        & (stones >> 4)
}

pub fn contains_five_in_a_row(stones: u16) -> bool {
    calculate_five_in_a_rows(stones) != 0
}

// 128 KiB
struct VectorMatchLut {
    black: [u8; u16::MAX as usize],
    white: [u8; u16::MAX as usize],
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ExtendedMatch {
    Left,
    Right
}

impl ExtendedMatch {

    const fn reverse(&self) -> Self {
        match self {
            Right => Left,
            Left => Right
        }
    }

}

#[derive(Copy, Clone, Debug)]
#[repr(align(32))]
struct SlicePatchData {
    patch_mask: u64,
    closed_four_clear_mask: u64,
    closed_four_mask: u64,
    extended_match: Option<ExtendedMatch>,
}

assert_struct_sizes!(SlicePatchData, size=32, align=32);

impl SlicePatchData {

    pub const fn merge(self, other: Self) -> Self {
        let extended_match = if self.extended_match.is_some() {
            self.extended_match
        } else {
            other.extended_match
        };

        let self_fours: [u8; 8] = self.closed_four_mask.to_ne_bytes();
        let other_fours: [u8; 8] = other.closed_four_mask.to_ne_bytes();
        let mut double_four_mask: [u8; 8] = [0; 8];

        const_for!(idx in 0, 8; {
            if self_fours[idx] == pattern::CLOSED_FOUR_SINGLE
                && other_fours[idx] == pattern::CLOSED_FOUR_SINGLE
            {
                double_four_mask[idx] = pattern::CLOSED_FOUR_DOUBLE;
            }
        });

        Self {
            patch_mask: self.patch_mask | other.patch_mask | u64::from_ne_bytes(double_four_mask),
            closed_four_clear_mask: self.closed_four_clear_mask ^ other.closed_four_clear_mask,
            closed_four_mask: self.closed_four_mask ^ other.closed_four_mask,
            extended_match
        }
    }

}

struct PatchLut {
    black: [SlicePatchData; 76],
    white: [SlicePatchData; 71],
    _dbg_black_length: usize,
    _dbg_white_length: usize,
}

struct SlicePatternLut {
    vector: VectorMatchLut,
    patch: PatchLut,
}

const SLICE_PATTERN_LUT: SlicePatternLut = build_slice_pattern_lut();

#[allow(unused_mut)]
const fn build_slice_pattern_lut() -> SlicePatternLut {
    let mut slice_pattern_lut = {
        let initial_patch_data = SlicePatchData {
            patch_mask: 0,
            closed_four_clear_mask: 0,
            closed_four_mask: 0,
            extended_match: None
        };

        SlicePatternLut {
            vector: VectorMatchLut {
                black: [0; u16::MAX as usize],
                white: [0; u16::MAX as usize],
            },
            patch: PatchLut {
                black: [initial_patch_data; 76],
                white: [initial_patch_data; 71],
                _dbg_black_length: 0,
                _dbg_white_length: 0,
            },
        }
    };

    let mut temp_vector_match_lut_black: [[u8; 4]; u16::MAX as usize] = [[0; 4]; u16::MAX as usize];
    let mut temp_vector_match_lut_white: [[u8; 4]; u16::MAX as usize] = [[0; 4]; u16::MAX as usize];

    let mut patch_top_black: usize = 0;
    let mut patch_top_white: usize = 0;

    macro_rules! fill_array {
        ($a:expr) => ([$a, "", "", ""]);
        ($a:expr,$b:expr) => ([$a, $b, "", ""]);
        ($a:expr,$b:expr,$c:expr) => ([$a, $b, $c, ""]);
        ($a:expr,$b:expr,$c:expr,$d:expr) => ([$a, $b, $c, $d]);
    }

    macro_rules! embed_pattern {
        (black,asymmetry,long-pattern,$direction:expr,$pattern:literal,$($patch:literal),+) => {
            embed_pattern!(black, rev=false, Some($direction), $pattern, fill_array!($($patch),+));
            embed_pattern!(black, rev=true, Some($direction.reverse()), $pattern, fill_array!($($patch),+));
        };
        ($color:ident,symmetry,$pattern:literal,$($patch:literal),+) => {
            embed_pattern!($color, rev=false, Option::None, $pattern, fill_array!($($patch),+));
        };
        ($color:ident,asymmetry,$pattern:literal,$($patch:literal),+) => {
            embed_pattern!($color, rev=false, Option::None, $pattern, fill_array!($($patch),+));
            embed_pattern!($color, rev=true, Option::None, $pattern, fill_array!($($patch),+));
        };
        (black,rev=$rev:expr,$extended_match:expr,$pattern:literal,$patches:expr) => {
            embed_pattern!(rev=$rev, temp_vector_match_lut_black, slice_pattern_lut.patch.black, patch_top_black, $extended_match, $pattern, $patches);
        };
        (white,rev=$rev:expr,$extended_match:expr,$pattern:literal,$patches:expr) => {
            embed_pattern!(rev=$rev, temp_vector_match_lut_white, slice_pattern_lut.patch.white, patch_top_white, $extended_match, $pattern, $patches);
        };
        (rev=$rev:expr,$vector_expr:expr,$patch_expr:expr,$patch_top_expr:expr,$extended_match:expr,$pattern:literal,$patches:expr) => {{
            $patch_top_expr += 1;
            $patch_expr[$patch_top_expr] = build_slice_patch_data($extended_match, $rev, $patches);

            let vector_variants = parse_vector_variant_literal($pattern, $rev);
            flash_vector_variants(&mut $vector_expr, $patch_top_expr, vector_variants, 0, 0);
        }};
    }

    // black-open-three

    embed_pattern!(black, asymmetry, "!.OO...!", "!.OO3..!", "!.OO.3.!");
    embed_pattern!(black, asymmetry, "X..OO..!", "X.3OO..!");
    embed_pattern!(black, asymmetry, "!.O.O..!", "!.O3O..!", "!.O.O3.!");
    embed_pattern!(black, symmetry, "!.O..O.!", "!.O3.O.!", "!.O.3O.!");
    embed_pattern!(black, asymmetry, long-pattern, Left, "..OO...O", "..OO3..O"); // [!]..OO...O

    // white-open-three

    embed_pattern!(white, asymmetry, ".OO...", ".OO.3.");
    embed_pattern!(white, asymmetry, "!.OO...", "!.OO3..");
    embed_pattern!(white, asymmetry, "X..OO..", "X.3OO..");
    embed_pattern!(white, asymmetry, ".O.O..!!", ".O.O3.!!");
    embed_pattern!(white, asymmetry, ".O.O..O!", ".O.O3.O!");
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
    embed_pattern!(black, asymmetry, long-pattern, Left, "..OOO..O", "..OOOF.O", "C.OOO..O"); // [!]..OOO..O

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

    slice_pattern_lut.patch._dbg_black_length =
        compress_pattern_lut(temp_vector_match_lut_black, &mut slice_pattern_lut.vector.black, &mut slice_pattern_lut.patch.black, patch_top_black);
    slice_pattern_lut.patch._dbg_white_length =
        compress_pattern_lut(temp_vector_match_lut_white, &mut slice_pattern_lut.vector.white, &mut slice_pattern_lut.patch.white, patch_top_white);

    slice_pattern_lut
}

const fn flash_vector_variant(
    lut: &mut [[u8; 4]; u16::MAX as usize],
    patch_pointer: usize,
    variants: VectorVariants,
    depth: usize,
    new_vector: u16,
) {
    if depth < 7 {
        flash_vector_variants(lut, patch_pointer, variants, depth + 1, new_vector)
    } else {
        let mut bucket = lut[new_vector as usize];

        const_for!(idx in 0, 4; {
            if bucket[idx] == 0 {
                bucket[idx] = patch_pointer as u8;
                lut[new_vector as usize] = bucket;
                return;
            }
        });
    }
}

const fn flash_vector_variants(
    lut: &mut [[u8; 4]; u16::MAX as usize],
    patch_pointer: usize,
    variants: VectorVariants,
    depth: usize,
    vector: u16,
) {
    if variants[depth].stone {
        let new_vector = (0b0000_0000_0000_0001 << depth) | vector;
        flash_vector_variant(lut, patch_pointer, variants, depth, new_vector);
    }

    if variants[depth].block {
        let new_vector = (0b0000_0001_0000_0000 << depth) | vector;
        flash_vector_variant(lut, patch_pointer, variants, depth, new_vector);
    }

    if variants[depth].empty {
        flash_vector_variant(lut, patch_pointer, variants, depth, vector);
    }
}

const fn compress_pattern_lut<const N: usize>(
    temp_vector: [[u8; 4]; u16::MAX as usize],
    vector: &mut [u8; u16::MAX as usize],
    patch: &mut [SlicePatchData; N],
    mut patch_top: usize
) -> usize {
    const_for!(idx in 0, temp_vector.len(); {
        let patch_pointer_bucket = temp_vector[idx];
        vector[idx] = if patch_pointer_bucket[1] != 0 {
            patch_top += 1;

            const_for!(i in 0, 4; {
                patch[patch_top] = patch[patch_top].merge(patch[patch_pointer_bucket[i] as usize]);
            });

            patch_top as u8
        } else {
            patch_pointer_bucket[0]
        };
    });

    patch_top
}


const fn build_slice_patch_data(extended_match: Option<ExtendedMatch>, reversed: bool, sources: [&str; 4]) -> SlicePatchData {
    let mut patch_mask: [u8; 8] = [0; 8];
    let mut closed_four_clear_mask: [u8; 8] = [0; 8];
    let mut closed_four_mask: [u8; 8] = [0; 8];

    const_for!(idx in 0, 4; {
        if !sources[idx].is_empty() {
            let (pos, kind) = parse_patch_literal(sources[idx], reversed);

            if kind == pattern::CLOSED_FOUR_SINGLE {
                closed_four_clear_mask[pos] = pattern::CLOSED_FOUR_DOUBLE;
                closed_four_mask[pos] = pattern::CLOSED_FOUR_SINGLE;
            } else {
                patch_mask[pos] |= kind;
            }
        }
    });

    SlicePatchData {
        patch_mask: u64::from_ne_bytes(patch_mask),
        closed_four_clear_mask: u64::from_ne_bytes(closed_four_clear_mask),
        closed_four_mask: u64::from_ne_bytes(closed_four_mask),
        extended_match,
    }
}

#[derive(Copy, Clone, Default)]
struct VectorVariantElement {
    stone: bool,
    block: bool,
    empty: bool,
}

type VectorVariants = [VectorVariantElement; 8];

const fn parse_vector_variant_literal(source: &str, reversed: bool) -> VectorVariants {
    let mut acc: VectorVariants = [VectorVariantElement { stone: true, block: true, empty: true }; 8];

    const_for!(idx in 0, source.len(); {
        let pos = if reversed { 7 - idx } else { idx };
        acc[pos] = match source.as_bytes()[idx] as char {
            'O' => VectorVariantElement { stone: true, block: false, empty: false },
            'X' => VectorVariantElement { stone: false, block: true, empty: false },
            '!' => VectorVariantElement { stone: false, block: true, empty: true },
            '.' => VectorVariantElement { stone: false, block: false, empty: true },
            _ => unreachable!(),
        };
    });

    acc
}

const fn parse_patch_literal(source: &str, reversed: bool) -> (usize, u8) {
    const_for!(idx in 0, source.len(); {
        let pos = if reversed { 7 - idx } else { idx };
        match source.as_bytes()[idx] as char {
            '3' => return (pos, pattern::OPEN_THREE),
            'C' => return (pos, pattern::CLOSE_THREE),
            '4' => return (pos, pattern::OPEN_FOUR),
            'F' => return (pos, pattern::CLOSED_FOUR_SINGLE),
            '5' => return (pos, pattern::FIVE),
            '6' => return (pos, pattern::OVERLINE),
            _ => {}
        }
    });

    unreachable!()
}
