use crate::cache::dummy_patch_cache::DummyPatchCache;
use crate::cache::patch_cache::PatchCache;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::forbidden_kind::ForbiddenKind;
use crate::notation::pos::cartesian_to_index;
use crate::notation::rule;
use crate::pattern::{FormationPatch, EMPTY_SLICE_PATH};
use crate::slice::Slice;

pub const CLOSED_FOUR_SINGLE_MASK: u8   = 0b1000_0000;
pub const CLOSED_FOUR_DOUBLE_MASK: u8   = 0b1100_0000;
pub const OPEN_FOUR_MASK: u8            = 0b0010_0000;
pub const TOTAL_FOUR_MASK: u8           = 0b1110_0000;
pub const FIVE_MASK: u8                 = 0b0001_0000;

pub const OPEN_THREE_MASK: u8           = 0b0000_1000;
pub const CLOSE_THREE_MASK: u8          = 0b0000_0100;
pub const CORE_THREE_MASK: u8           = 0b0000_0010;
pub const OVERLINE_MASK: u8             = 0b0000_0001;

const U32_CLOSED_FOUR_MASK: u32     = 0b1100_0000__1100_0000__1100_0000__1100_0000;
const U32_OPEN_FOUR_MASK: u32     = 0b0010_0000__0010_0000__0010_0000__0010_0000;
const U32_TOTAL_FOUR_MASK: u32     = 0b1110_0000__1110_0000__1110_0000__1110_0000;
const U32_FIVE_MASK: u32           = 0b0001_0000__0001_0000__0001_0000__0001_0000;

const U32_OPEN_THREE_MASK: u32     = 0b0000_1000__0000_1000__0000_1000__0000_1000;
const U32_CLOSE_THREE_MASK: u32    = 0b0000_0100__0000_0100__0000_0100__0000_0100;
const U32_CORE_THREE_MASK: u32    = 0b0000_0010__0000_0010__0000_0010__0000_0010;

const U32_OVERLINE_MASK: u32       = 0b0000_0001__0000_0001__0000_0001__0000_0001;

// encoded in 8-bit: closed-4-1 closed-4-2 open-4 five _ open-3 close-3 core-3 overline
// total 32bit
#[derive(Debug, Copy, Clone)]
pub struct FormationUnit {
    horizontal: u8,
    vertical: u8,
    ascending: u8,
    descending: u8
}

impl Default for FormationUnit {

    fn default() -> Self {
        Self {
            horizontal: 0,
            vertical: 0,
            ascending: 0,
            descending: 0
        }
    }

}

impl FormationUnit {

    pub fn is_empty(&self) -> bool {
        let raw: u32 = unsafe { std::mem::transmute(*self) };
        raw == 0
    }

    pub fn open_three_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(OPEN_THREE_MASK)
    }

    pub fn close_three_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(CLOSE_THREE_MASK)
    }

    pub fn closed_four_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(CLOSED_FOUR_SINGLE_MASK)
    }

    pub fn open_four_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(OPEN_FOUR_MASK)
    }

    pub fn five_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(FIVE_MASK)
    }

    fn with_mask_at<const D: Direction>(&self, mask: u8) -> bool {
        let encoded = match D {
            Direction::Horizontal => self.horizontal,
            Direction::Vertical => self.vertical,
            Direction::Ascending => self.ascending,
            Direction::Descending => self.descending
        };

        encoded & mask == mask
    }

    pub fn count_open_threes(&self) -> u32 {
        self.with_mask(U32_OPEN_THREE_MASK).count_ones()
    }

    pub fn count_close_threes(&self) -> u32 {
        self.with_mask(U32_CLOSE_THREE_MASK).count_ones()
    }

    pub fn count_core_threes(&self) -> u32 {
        self.with_mask(U32_CORE_THREE_MASK).count_ones()
    }

    pub fn count_closed_fours(&self) -> u32 {
        self.with_mask(U32_CLOSED_FOUR_MASK).count_ones()
    }

    pub fn count_open_fours(&self) -> u32 {
        self.with_mask(U32_OPEN_FOUR_MASK).count_ones()
    }

    pub fn count_fours(&self) -> u32 {
        self.with_mask(U32_TOTAL_FOUR_MASK).count_ones()
    }

    pub fn count_fives(&self) -> u32 {
        self.with_mask(U32_FIVE_MASK).count_ones()
    }

    pub fn has_five(&self) -> bool {
        self.with_mask(U32_FIVE_MASK) != 0
    }

    pub fn has_overline(&self) -> bool {
        self.with_mask(U32_OVERLINE_MASK) != 0
    }

    fn with_mask(&self, mask: u32) -> u32 {
        unsafe { std::mem::transmute::<_, u32>(*self) & mask }
    }

}

#[derive(Debug, Copy, Clone)]
pub struct Formation {
    pub black_formation: FormationUnit,
    pub white_formation: FormationUnit
}

impl Default for Formation {

    fn default() -> Self {
        Self {
            black_formation: FormationUnit::default(),
            white_formation: FormationUnit::default()
        }
    }

}

impl Formation {

    #[inline(always)]
    pub fn access_unit<const C: Color>(&self) -> &FormationUnit {
        match C {
            Color::Black => &self.black_formation,
            Color::White => &self.white_formation
        }
    }

    pub fn is_empty(&self) -> bool {
        let raw: u64 = unsafe { std::mem::transmute::<_, u64>(*self) };
        raw == 0
    }

    pub fn is_forbidden(&self) -> bool {
        self.is_empty() && (
            self.black_formation.count_open_threes() > 1
                || self.black_formation.count_fours() > 1
                || self.black_formation.has_overline()
        ) && !self.black_formation.has_five()
    }

    pub fn forbidden_kind(&self) -> Option<ForbiddenKind> {
        let raw: u32 = unsafe { std::mem::transmute(self.black_formation) };

        if raw == 0 || self.black_formation.has_five() {
            None
        } else if self.black_formation.count_open_threes() > 1 {
            Some(ForbiddenKind::DoubleThree)
        } else if self.black_formation.count_fours() > 1 {
            Some(ForbiddenKind::DoubleFour)
        } else if self.black_formation.has_overline() {
            Some(ForbiddenKind::Overline)
        } else {
            None
        }
    }

    pub fn apply_mask_mut<const D: Direction>(mut self, patch: FormationPatch) {
        match D {
            Direction::Horizontal => {
                self.black_formation.vertical = patch.black_patch;
                self.white_formation.vertical = patch.white_patch;
            },
            Direction::Vertical => {
                self.black_formation.horizontal = patch.black_patch;
                self.white_formation.horizontal = patch.white_patch;
            },
            Direction::Ascending => {
                self.black_formation.ascending = patch.black_patch;
                self.white_formation.ascending = patch.white_patch;
            },
            Direction::Descending => {
                self.black_formation.descending = patch.black_patch;
                self.white_formation.descending = patch.white_patch;
            }
        }
    }

}

#[derive(Debug, Copy, Clone)]
pub struct Formations(pub [Formation; rule::BOARD_SIZE]);

impl Default for Formations {

    fn default() -> Self {
        Self([Formation::default(); rule::BOARD_SIZE])
    }

}

impl Formations {

    #[inline(always)]
    pub fn update_with_slice_mut<const D: Direction>(&mut self, slice: &Slice) {
        let mut patch_cache = DummyPatchCache {}; // TODO: DEBUG
        let slice_patch = if let Some(slice_patch) = patch_cache.probe_mut(slice.slice_key()) {
            slice_patch
        } else {
            let patch = slice.calculate_slice_patch();
            patch_cache.put_mut(slice.slice_key(), patch);
            patch
        };

        if slice_patch == EMPTY_SLICE_PATH {
            return
        }

        for offset in 0 .. slice.length {
            let idx = match D {
                Direction::Horizontal =>
                    cartesian_to_index(slice.start_pos.row(), slice.start_pos.col() + offset) as usize,
                Direction::Vertical =>
                    cartesian_to_index(slice.start_pos.row() + offset, slice.start_pos.col()) as usize,
                Direction::Ascending =>
                    cartesian_to_index(slice.start_pos.row() + offset, slice.start_pos.col() + offset) as usize,
                Direction::Descending =>
                    cartesian_to_index(slice.start_pos.row() - offset, slice.start_pos.col() + offset) as usize,
            };

            self.0[idx].apply_mask_mut::<D>(slice_patch[offset as usize])
        }
    }

}
