use crate::notation::direction::Direction;
use crate::notation::forbidden_kind::ForbiddenKind;
use crate::notation::pos::Pos;
use crate::notation::rule;
use crate::pattern::FormationPatch;
use crate::slice::Slice;

const CLOSED_FOUR_SINGLE_MASK: u8   = 0b1000_0000;
const CLOSED_FOUR_DOUBLE_MASK: u8   = 0b1100_0000;
const OPEN_FOUR_MASK: u8            = 0b0010_0000;
const TOTAL_FOUR_MASK: u8           = 0b1110_0000;

const OPEN_THREE_MASK: u8           = 0b0000_1000;
const CLOSE_THREE_MASK: u8          = 0b0000_0100;
const FIVE_MASK: u8                 = 0b0000_0010;
const OVERLINE_MASK: u8             = 0b0000_0001;

const UNIT_TOTAL_FOUR_MASK: u32     = 0b1110_0000__1110_0000__1110_0000__1110_0000;

const UNIT_OPEN_THREE_MASK: u32     = 0b0000_1000__0000_1000__0000_1000__0000_1000;
const UNIT_CLOSE_THREE_MASK: u32    = 0b0000_0100__0000_0100__0000_0100__0000_0100;
const UNIT_FIVE_MASK: u32           = 0b0000_0010__0000_0010__0000_0010__0000_0010;
const UNIT_OVERLINE_MASK: u32       = 0b0000_0001__0000_0001__0000_0001__0000_0001;

// 8-bit: closed-4-1 closed-4-2 open-4 padding _ open-3 close-3 five overline
#[derive(Debug, Copy, Clone)]
pub struct FormationUnit {
    pub horizontal: u8,
    pub vertical: u8,
    pub ascending: u8,
    pub descending: u8
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

    pub fn open_three_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(OPEN_THREE_MASK)
    }

    pub fn close_three_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(CLOSE_THREE_MASK)
    }

    pub fn open_four_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(OPEN_FOUR_MASK)
    }

    pub fn five_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(FIVE_MASK)
    }

    pub fn closed_four_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(CLOSED_FOUR_SINGLE_MASK)
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
        self.with_mask(UNIT_OPEN_THREE_MASK).count_ones()
    }

    pub fn count_close_threes(&self) -> u32 {
        self.with_mask(UNIT_CLOSE_THREE_MASK).count_ones()
    }

    pub fn count_fours(&self) -> u32 {
        self.with_mask(UNIT_TOTAL_FOUR_MASK).count_ones()
    }

    pub fn has_five(&self) -> bool {
        self.with_mask(UNIT_FIVE_MASK) > 0
    }

    pub fn has_overline(&self) -> bool {
        self.with_mask(UNIT_OVERLINE_MASK) > 0
    }

    fn with_mask(&self, mask: u32) -> u32 {
        unsafe { std::mem::transmute::<_, u32>(*self) & mask }
    }

}

#[derive(Debug, Copy, Clone)]
pub struct Formation {
    pub black_formation: FormationUnit,
    pub white_formation: FormationUnit,
}

impl Default for Formation {

    fn default() -> Self {
        Self {
            black_formation: FormationUnit::default(),
            white_formation: FormationUnit::default(),
        }
    }

}

impl Formation {

    pub fn is_forbidden(&self) -> bool {
        false
    }

    pub fn forbidden_kind(&self) -> Option<ForbiddenKind> {
        if self.black_formation.count_open_threes() > 1 {
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
    pub fn update_with_slice_mut<const D: Direction, F>(&mut self, slice: &Slice, build_idx: F)
    where F: Fn(usize, &Pos) -> usize
    {
        for (offset, mask) in slice.calculate_slice_patch().into_iter().enumerate() {
            let idx = build_idx(offset, &slice.start_pos);
            self.0[idx].apply_mask_mut::<D>(mask);
        }
    }

}
