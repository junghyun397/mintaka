use crate::notation::rule;
use crate::slice::Slice;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct FormationPatch {
    pub black_patch: u8,
    pub white_patch: u8
}

pub type SlicePatch = [FormationPatch; rule::U_BOARD_WIDTH];

pub const EMPTY_SLICE_PATH: SlicePatch = [FormationPatch { black_patch: 0, white_patch: 0}; rule::U_BOARD_WIDTH];

impl Slice {

    pub fn calculate_slice_patch(&self) -> SlicePatch {
        EMPTY_SLICE_PATCH
    }

}

fn find_unidirectional_patterns(acc: &mut SlicePatch, idx: usize, black_stones: u8, white_stones: u8, wall: u8) {
    let black_ones = black_stones.count_ones();
    let white_ones = white_stones.count_ones();

    if black_ones < 2 || white_ones < 2 {
        return
    }
    todo!()
}

fn find_bidirectional_patterns(acc: &mut SlicePatch, idx: usize, black_stones: u8, white_stones: u8, wall: u8) {
    todo!()
}

fn increase_closed_four(packed: u8) -> u8 {
    packed | (0b1000_000 >> (packed >> 7))
}

fn mark_blind_three(packed: u8) -> u8 {
    packed | 0b1
}

fn unmark_blind_three(packed: u8) -> u8 {
    packed & !0b1
}

macro_rules! fp {
    ($b:literal,$w:literal) => {
        FormationPatch {
            black_patch: $b,
            white_patch: $w
        }
    };
}

const EMPTY_SLICE_PATCH: SlicePatch = [
    fp!(0, 0), fp!(0, 0), fp!(0, 0), fp!(0, 0), fp!(0, 0),
    fp!(0, 0), fp!(0, 0), fp!(0, 0), fp!(0, 0), fp!(0, 0),
    fp!(0, 0), fp!(0, 0), fp!(0, 0), fp!(0, 0), fp!(0, 0),
];
