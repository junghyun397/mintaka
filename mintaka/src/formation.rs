use crate::notation::direction::Direction;
use crate::notation::forbidden_kind::ForbiddenKind;
use crate::notation::pos::Pos;
use crate::notation::rule;
use crate::slice::Slice;

const OPEN_THREE_MASK: u16 = 0b1111_0000_0000_0000;
const CLOSE_THREE_MASK: u16 = 0b0000_1111_0000_0000;
const OPEN_FOUR_MASK: u16 = 0b0000_0000_1111_0000;
const FIVE_MASK: u16 = 0b0000_0000_0000_1111;

// 32-bit
#[derive(Debug, Copy, Clone)]
pub struct FormationUnit {
    pub o3_c3_o4_5: u16,
    pub closed_four: u8,
    pub etc: u8,
}

impl Default for FormationUnit {

    fn default() -> Self {
        Self {
            o3_c3_o4_5: 0,
            closed_four: 0,
            etc: 0
        }
    }

}

impl FormationUnit {

    fn count_open_threes(&self) -> u32 {
        (self.o3_c3_o4_5 & OPEN_THREE_MASK).count_ones()
    }

    fn count_close_threes(&self) -> u32 {
        (self.o3_c3_o4_5 & CLOSE_THREE_MASK).count_ones()
    }

    fn count_open_fours(&self) -> u32 {
        (self.o3_c3_o4_5 & OPEN_FOUR_MASK).count_ones()
    }

    fn count_fives(&self) -> u32 {
        (self.o3_c3_o4_5 & FIVE_MASK).count_ones()
    }

    fn count_closed_fours(&self) -> u32 {
        self.closed_four.count_ones()
    }

}

const U64_MASK: u64 = 0b1000_1000_1000_1000__1000_1000_1000_1000__1000_1000_1000_1000__1000_1000_1000_1000;

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
        if (self.black_formation.o3_c3_o4_5 & OPEN_THREE_MASK).count_ones() > 1 {
            Some(ForbiddenKind::DoubleThree)
        } else if (self.black_formation.o3_c3_o4_5 & OPEN_FOUR_MASK).count_ones()
            + self.black_formation.closed_four.count_ones() > 1 {
            Some(ForbiddenKind::DoubleFour)
        } else {
            None
        }
    }

    pub fn apply_mask(self, direction: Direction, mask: Self) -> Self {
        unsafe {
            let mut mask_raw: u64 = std::mem::transmute(mask);
            mask_raw = mask_raw >> direction as usize;
            let mut self_raw: u64 = std::mem::transmute(self);
            self_raw = self_raw & !(U64_MASK >> direction as usize) | mask_raw;
            std::mem::transmute(self_raw)
        }
    }

}

pub type FormationLine = [Formation; rule::U_BOARD_WIDTH];

#[derive(Debug, Copy, Clone)]
pub struct Formations(pub [Formation; rule::BOARD_SIZE]);

impl Default for Formations {

    fn default() -> Self {
        Self([Formation::default(); rule::BOARD_SIZE])
    }

}

impl Formations {

    #[inline(always)]
    pub fn update_with_slice_mut<F>(&mut self, slice: &Slice, direction: Direction, build_idx: F)
    where F: Fn(usize, &Pos) -> usize
    {
        for (offset, mask) in slice.calculate_formation_masks().into_iter().enumerate() {
            let idx = build_idx(offset, &slice.start_pos);
            self.0[idx] = self.0[idx].apply_mask(direction, mask);
        }
    }

}

pub mod preset {
    use crate::formation::FormationUnit;

    pub const OPEN_THREE: FormationUnit = FormationUnit {
        o3_c3_o4_5: 0b1000_0000_0000_0000,
        closed_four: 0b0,
        etc: 0b0,
    };

    pub const CLOSE_THREE: FormationUnit = FormationUnit {
        o3_c3_o4_5: 0b0000_1000_0000_0000,
        closed_four: 0b0,
        etc: 0b0,
    };

    pub const OPEN_FOUR: FormationUnit = FormationUnit {
        o3_c3_o4_5: 0b0000_0000_1000_0000,
        closed_four: 0b0,
        etc: 0b0,
    };

    pub const FIVE: FormationUnit = FormationUnit {
        o3_c3_o4_5: 0b0000_0000_0000_1000,
        closed_four: 0b0,
        etc: 0b0,
    };

    pub const CLOSED_FOUR_SINGLE: FormationUnit = FormationUnit {
        o3_c3_o4_5: 0b0000_0000_0000_0000,
        closed_four: 0b1000_0000,
        etc: 0b0,
    };

    pub const CLOSED_FOUR_DOUBLE: FormationUnit = FormationUnit {
        o3_c3_o4_5: 0b0000_0000_0000_0000,
        closed_four: 0b1000_1000,
        etc: 0b0,
    };

    pub const OVERLINE: FormationUnit = FormationUnit {
        o3_c3_o4_5: 0b0000_0000_0000_0000,
        closed_four: 0b0000_0000,
        etc: 0b1000_0000,
    };

}
