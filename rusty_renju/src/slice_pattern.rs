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
        let extended_stones: usize = (self.stones.get::<C>() as usize) << 3;
        let extended_blocks: usize = (self.blocks::<C>() as usize) << 3 | 0b111;

        let mut acc: SlicePattern = SlicePattern::EMPTY;
        for shift in 0 .. self.length as usize - 1 {
            let stones = (extended_stones >> shift) & 0x00FF;
            let blocks = (extended_blocks >> shift) & 0x00FF;

            lookup_patterns::<R, C>(
                &mut acc,
                shift,
                encode_vector_match_key(stones, blocks),
                extended_stones,
            );
        }

        acc
    }

}

#[inline(always)]
fn lookup_patterns<const R: RuleKind, const C: Color>(
    acc: &mut SlicePattern,
    shift: usize,
    key: usize,
    raw: usize,
) {
    #[cold]
    fn extended_match_for_black(direction: ExtendedMatch, b_raw: usize, shift: usize) -> bool {
        match direction {
            Left => b_raw & (0b1 << shift.saturating_sub(1)) == 0,
            Right => b_raw & (0b1 << (shift + 8)) == 0
        }
    }

    let patch_pointer = match (R, C) {
        (RuleKind::Renju, Color::Black) => SLICE_PATTERN_LUT.vector.black[key],
        _ => SLICE_PATTERN_LUT.vector.white[key],
    };

    if patch_pointer != 0 {
        let slice_patch_data = unsafe { match (R, C) {
            (RuleKind::Renju, Color::Black) => SLICE_PATTERN_LUT.patch.black.get_unchecked(patch_pointer as usize),
            _ => SLICE_PATTERN_LUT.patch.white.get_unchecked(patch_pointer as usize),
        } };

        if R == RuleKind::Renju && C == Color::Black
            && let Some(extended_match) = slice_patch_data.extended_match
            && !extended_match_for_black(extended_match, raw, shift)
        {
            return;
        }

        let lane_shift = (shift as isize - 3) * 8;

        if slice_patch_data.patch_mask != 0 {
            acc.patterns |= shift_and_fit_u128(slice_patch_data.patch_mask, lane_shift);
        }

        if slice_patch_data.closed_four_mask != 0 {
            acc.patterns |= increase_closed_four(
                acc.patterns,
                shift_and_fit_u128(slice_patch_data.closed_four_clear_mask, lane_shift),
                shift_and_fit_u128(slice_patch_data.closed_four_mask, lane_shift)
            );
        }
    }
}

#[inline(always)]
fn shift_and_fit_u128(x: u64, shift: isize) -> u128 {
    if shift < 0 {
        (x as u128) >> -shift
    } else {
        (x as u128) << shift
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

const VECTOR_MATCH_LUT_SIZE: usize = u16::MAX as usize + 1;

// attempted 3^8 + 1 lookup table using base-3 and nibble-based indexing,
// but observed over 60% slowdown due to pipeline stalls.
// to avoid pipeline stalls, use simpler indexing.
#[inline]
const fn encode_vector_match_key(stones: usize, blocks: usize) -> usize {
    (blocks << 8) | stones
}

// 128 KiB
#[repr(align(32))]
struct VectorMatchLut {
    black: [u8; VECTOR_MATCH_LUT_SIZE],
    white: [u8; VECTOR_MATCH_LUT_SIZE],
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

        let self_fours: [u8; 8] = self.closed_four_mask.to_le_bytes();
        let other_fours: [u8; 8] = other.closed_four_mask.to_le_bytes();
        let mut double_four_mask: [u8; 8] = [0; 8];

        const_for!(idx in 0, 8; {
            if self_fours[idx] == pattern::CLOSED_FOUR_SINGLE
                && other_fours[idx] == pattern::CLOSED_FOUR_SINGLE
            {
                double_four_mask[idx] = pattern::CLOSED_FOUR_DOUBLE;
            }
        });

        Self {
            patch_mask: self.patch_mask | other.patch_mask | u64::from_le_bytes(double_four_mask),
            closed_four_clear_mask: self.closed_four_clear_mask ^ other.closed_four_clear_mask,
            closed_four_mask: self.closed_four_mask ^ other.closed_four_mask,
            extended_match
        }
    }

}

const BLACK_LUT_SIZE: usize = 97;
const WHITE_LUT_SIZE: usize = 92;

#[repr(align(32))]
struct PatchLut {
    black: [SlicePatchData; BLACK_LUT_SIZE],
    white: [SlicePatchData; WHITE_LUT_SIZE],
}

struct SlicePatternLut {
    vector: VectorMatchLut,
    patch: PatchLut,
    _dbg_black_length: usize,
    _dbg_white_length: usize,
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
                black: [0; VECTOR_MATCH_LUT_SIZE],
                white: [0; VECTOR_MATCH_LUT_SIZE],
            },
            patch: PatchLut {
                black: [initial_patch_data; BLACK_LUT_SIZE],
                white: [initial_patch_data; WHITE_LUT_SIZE],
            },
            _dbg_black_length: 0,
            _dbg_white_length: 0,
        }
    };

    let mut temp_vector_match_lut_black: [[u8; 4]; VECTOR_MATCH_LUT_SIZE] = [[0; 4]; VECTOR_MATCH_LUT_SIZE];
    let mut temp_vector_match_lut_white: [[u8; 4]; VECTOR_MATCH_LUT_SIZE] = [[0; 4]; VECTOR_MATCH_LUT_SIZE];

    let mut patch_top_black: usize = 0;
    let mut patch_top_white: usize = 0;

    macro_rules! fill_array {
        ($a:expr) => ([$a, "", "", ""]);
        ($a:expr,$b:expr) => ([$a, $b, "", ""]);
        ($a:expr,$b:expr,$c:expr) => ([$a, $b, $c, ""]);
        ($a:expr,$b:expr,$c:expr,$d:expr) => ([$a, $b, $c, $d]);
    }

    macro_rules! embed_pattern {
        (both,$mirror:ident,$pattern:literal,$($patch:literal),+) => {
            embed_pattern!(black, $mirror, $pattern, $($patch),+);
            embed_pattern!(white, $mirror, $pattern, $($patch),+);
        };
        (black,asymmetry,long-pattern,$direction:expr,$pattern:literal,$($patch:literal),+) => {
            flash_pattern!(black, rev=false, Some($direction), $pattern, fill_array!($($patch),+));
            flash_pattern!(black, rev=true, Some($direction.reverse()), $pattern, fill_array!($($patch),+));
        };
        ($color:ident,symmetry,$pattern:literal,$($patch:literal),+) => {
            flash_pattern!($color, rev=false, Option::None, $pattern, fill_array!($($patch),+));
        };
        ($color:ident,asymmetry,$pattern:literal,$($patch:literal),+) => {
            flash_pattern!($color, rev=false, Option::None, $pattern, fill_array!($($patch),+));
            flash_pattern!($color, rev=true, Option::None, $pattern, fill_array!($($patch),+));
        };
    }

    macro_rules! flash_pattern {
        (black,rev=$rev:expr,$extended_match:expr,$pattern:literal,$patches:expr) => {
            flash_pattern!(rev=$rev, temp_vector_match_lut_black, slice_pattern_lut.patch.black, patch_top_black, $extended_match, $pattern, $patches);
        };
        (white,rev=$rev:expr,$extended_match:expr,$pattern:literal,$patches:expr) => {
            flash_pattern!(rev=$rev, temp_vector_match_lut_white, slice_pattern_lut.patch.white, patch_top_white, $extended_match, $pattern, $patches);
        };
        (rev=$rev:expr,$vector_expr:expr,$patch_expr:expr,$patch_top_expr:expr,$extended_match:expr,$pattern:literal,$patches:expr) => {{
            $patch_top_expr += 1;
            $patch_expr[$patch_top_expr] = build_slice_patch_data($extended_match, $rev, $patches);

            let vector_variants = parse_vector_variant_literal($pattern, $rev);
            flash_vector_variants(&mut $vector_expr, $patch_top_expr, vector_variants, 0, 0, 0);
        }};
    }

    // potential

    embed_pattern!(both, symmetry, "!O...O!", "!OP..O!", "!O.P.O!", "!O..PO!");

    embed_pattern!(both, asymmetry, "!..O...", "!.PO...", "!..OP..", "!..O.P.");
    embed_pattern!(both, asymmetry, "XO..O...", "XO..O.P.");
    embed_pattern!(both, asymmetry, "X.O....!", "X.OP...!", "X.O.P..!");
    embed_pattern!(both, asymmetry, "X.O....!", "X.O..P..");

    embed_pattern!(both, asymmetry, "XOO...!", "XOOP..!", "XOO.P.!", "XOO..P!");
    embed_pattern!(both, asymmetry, "XO.O..!", "XOPO..!", "XO.OP.!", "XO.O.P!");
    embed_pattern!(both, asymmetry, "XO..O.!", "XOP.O.!", "XO.PO.!", "XO..OP!");

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

    slice_pattern_lut._dbg_black_length = compress_pattern_lut(
        temp_vector_match_lut_black,
        &mut slice_pattern_lut.vector.black,
        &mut slice_pattern_lut.patch.black,
        patch_top_black
    );

    slice_pattern_lut._dbg_white_length = compress_pattern_lut(
        temp_vector_match_lut_white,
        &mut slice_pattern_lut.vector.white,
        &mut slice_pattern_lut.patch.white,
        patch_top_white
    );

    slice_pattern_lut
}

const fn flash_vector_variant(
    lut: &mut [[u8; 4]; VECTOR_MATCH_LUT_SIZE],
    patch_pointer: usize,
    variants: VectorVariants,
    depth: usize,
    stones: u16,
    blocks: u16
) {
    if depth < 7 {
        flash_vector_variants(lut, patch_pointer, variants, depth + 1, stones, blocks)
    } else {
        let lut_key = encode_vector_match_key(stones as usize, blocks as usize);

        let mut bucket = lut[lut_key];

        const_for!(idx in 0, 4; {
            if bucket[idx] == 0 {
                bucket[idx] = patch_pointer as u8;
                lut[lut_key] = bucket;
                return;
            }
        });
    }
}

const fn flash_vector_variants(
    lut: &mut [[u8; 4]; VECTOR_MATCH_LUT_SIZE],
    patch_pointer: usize,
    variants: VectorVariants,
    depth: usize,
    stones: u16,
    blocks: u16
) {
    if variants[depth].stone {
        let new_stones = (0b1 << depth) | stones;
        flash_vector_variant(lut, patch_pointer, variants, depth, new_stones, blocks);
    }

    if variants[depth].block {
        let new_blocks = (0b1 << depth) | blocks;
        flash_vector_variant(lut, patch_pointer, variants, depth, stones, new_blocks);
    }

    if variants[depth].empty {
        flash_vector_variant(lut, patch_pointer, variants, depth, stones, blocks);
    }
}

const fn compress_pattern_lut<const N: usize>(
    temp_vector: [[u8; 4]; VECTOR_MATCH_LUT_SIZE],
    vector: &mut [u8; VECTOR_MATCH_LUT_SIZE],
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
        patch_mask: u64::from_le_bytes(patch_mask),
        closed_four_clear_mask: u64::from_le_bytes(closed_four_clear_mask),
        closed_four_mask: u64::from_le_bytes(closed_four_mask),
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
            'P' => return (pos, pattern::POTENTIAL),
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
