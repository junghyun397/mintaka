use crate::notation::direction::Direction;
use crate::notation::forbidden_kind::ForbiddenKind;
use crate::notation::rule;

// 4-bit open-three | 4-bit close-three | 4-bit open-four | 4-bit five | 8-bit closed-four | total 24 bits
// 0000               0000                0000              0000         00000000
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FormationKind {
    OpenThree = 0,
    CloseThree = 4,
    ClosedFour = 8,
    Five = 12,
    OpenFour = 24
}

#[derive(Debug, Copy, Clone)]
pub struct Formation {
    pub o3_c3_o4_5: u16,
    pub closed_four: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct Cell {
    pub black_formation: Formation,
    pub white_formation: Formation,
    pub forbidden_kind: Option<ForbiddenKind>,
}

#[derive(Debug, Copy, Clone)]
pub struct Cells([Cell; rule::BOARD_SIZE]);

pub type FormationPairLine = [Cell; rule::U_BOARD_WIDTH];

impl Default for Formation {

    fn default() -> Self {
        Self {
            o3_c3_o4_5: 0,
            closed_four: 0,
        }
    }

}

impl Default for Cell {

    fn default() -> Self {
        Self {
            black_formation: Default::default(),
            white_formation: Default::default(),
        }
    }

}

impl Default for Cells {

    fn default() -> Self {
        Self([Default::default(); rule::BOARD_SIZE])
    }

}

impl Formation {

    fn formation_at(&self, direction: Direction) -> FormationKind {
        todo!()
    }

    fn count_open_threes(&self) -> u32 {
        (self.o3_c3_o4_5 & 0b1111_0000_0000_0000).count_ones()
    }

    fn count_close_threes(&self) -> u32 {
        (self.o3_c3_o4_5 & 0b0000_1111_0000_0000).count_ones()
    }

    fn count_open_fours(&self) -> u32 {
        (self.o3_c3_o4_5 & 0b0000_0000_1111_0000).count_ones()
    }

    fn count_fives(&self) -> u32 {
        (self.o3_c3_o4_5 & 0b0000_0000_0000_1111).count_ones()
    }

    fn count_closed_fours(&self) -> u32 {
        self.closed_four.count_ones()
    }

    fn apply_mask(&self, direction: Direction, mask: Formation) -> Self {
        let open_three_close_three_open_four_five_mask = mask.o3_c3_o4_5 >> direction as usize;
        let closed_four_mask = mask.closed_four >> direction as usize;

        todo!()
    }

}

pub mod preset {
    use crate::formation::Formation;

    pub const OPEN_THREE: Formation = Formation {
        o3_c3_o4_5: 0b1000_0000_0000_0000,
        closed_four: 0b0,
    };

    pub const CLOSE_THREE: Formation = Formation {
        o3_c3_o4_5: 0b0000_1000_0000_0000,
        closed_four: 0b0,
    };

    pub const OPEN_FOUR: Formation = Formation {
        o3_c3_o4_5: 0b0000_0000_1000_0000,
        closed_four: 0b0,
    };

    pub const FIVE: Formation = Formation {
        o3_c3_o4_5: 0b0000_0000_0000_1000,
        closed_four: 0b0,
    };

    pub const CLOSED_FOUR_SINGLE: Formation = Formation {
        o3_c3_o4_5: 0b0000_0000_0000_0000,
        closed_four: 0b1000_0000,
    };

    pub const CLOSED_FOUR_DOUBLE: Formation = Formation {
        o3_c3_o4_5: 0b0000_0000_0000_0000,
        closed_four: 0b1000_1000,
    };

}
