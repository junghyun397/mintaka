use crate::max;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::notation::pos::Pos;

const DIAGONAL_SLICE_AMOUNT: usize = pos::U_BOARD_WIDTH * 2 - 4 - 4 - 1;
const I_DIAGONAL_SLICE_AMOUNT: isize = DIAGONAL_SLICE_AMOUNT as isize;

#[derive(Debug, Copy, Clone)]
pub struct Slice {
    pub length: u8,
    pub start_row: u8,
    pub start_col: u8,
    pub black_stones: u16,
    pub white_stones: u16,
    pub black_pattern_available: bool,
    pub white_pattern_available: bool,
}

impl Slice {

    pub const PLACEHOLDER: Self = Self {
        length: 0,
        start_row: 0,
        start_col: 0,
        black_stones: 0,
        white_stones: 0,
        black_pattern_available: false,
        white_pattern_available: false,
    };

    pub const fn empty(length: u8, start_row: u8, start_col: u8) -> Self {
        Slice {
            length,
            start_row,
            start_col,
            black_stones: 0,
            white_stones: 0,
            black_pattern_available: false,
            white_pattern_available: false,
        }
    }

    pub fn set(mut self, color: Color, idx: u8) -> Self {
        self.set_mut(color, idx);
        self
    }

    pub fn unset(mut self, color: Color, idx: u8) -> Self {
        self.unset_mut(color, idx);
        self
    }

    pub fn set_mut(&mut self, color: Color, idx: u8) {
        let mask = 0b1 << idx;
        match color {
            Color::Black => self.black_stones |= mask,
            Color::White => self.white_stones |= mask
        };

        self.calculate_available_pattern_mut();
    }

    pub fn unset_mut(&mut self, color: Color, idx: u8) {
        let mask = !(0b1 << idx);
        match color {
            Color::Black => self.black_stones &= mask,
            Color::White => self.white_stones &= mask
        };

        self.calculate_available_pattern_mut();
    }

    pub fn stones<const C: Color>(&self) -> u16 {
        match C {
            Color::Black => self.black_stones,
            Color::White => self.white_stones
        }
    }

    pub fn stones_reversed_color<const C: Color>(&self) -> u16 {
        match C {
            Color::Black => self.white_stones,
            Color::White => self.black_stones
        }
    }

    pub fn pattern_available<const C: Color>(&self) -> bool {
        match C {
            Color::Black => self.black_pattern_available,
            Color::White => self.white_pattern_available
        }
    }

    pub fn stone_kind(&self, idx: u8) -> Option<Color> {
        let mask = 0b1 << idx;

        if self.black_stones & mask == mask {
            Some(Color::Black)
        } else if self.white_stones & mask == mask {
            Some(Color::White)
        } else {
            None
        }
    }

    pub fn calculate_idx(&self, direction: Direction, pos: Pos) -> usize {
        match direction {
            Direction::Vertical => pos.row_usize(),
            Direction::Horizontal => pos.col_usize(),
            _ => pos.col_usize() - self.start_col as usize
        }
    }

    pub fn remove_enclosed_stones(&self, mut p: u16, q: u16) -> u16 {
        // filter X O O O X, filter X O O O O X
        let mut vector = q as u32 | ((p as u32) << 16);

        const THREE_ENCLOSED: u32   = 0b0000_1110_0000_0000_0001_0001;
        const CLEAR_THREE_ENCLOSED: u16                    = 0b0_1110;
        const FOUR_ENCLOSED: u32    = 0b0001_1110_0000_0000_0010_0001;
        const CLEAR_FOUR_ENCLOSED: u16                    = 0b01_1110;

        for shift in 0 .. 10 {
            p &= if vector & THREE_ENCLOSED == THREE_ENCLOSED {
                !(CLEAR_THREE_ENCLOSED << shift)
            } else if vector & FOUR_ENCLOSED == FOUR_ENCLOSED {
                !(CLEAR_FOUR_ENCLOSED << shift)
            } else {
                u16::MAX
            };

            vector >>= 1;
        }

        p
    }

    fn calculate_available_pattern_mut(&mut self) {
        // filter . . O . . . .
        // filter O X . . O X .
        // filter O . . . O . .
        macro_rules! calculate_available_pattern {
            ($p:expr,$q:expr) => {{
                $p.count_ones() > 1
                    && $p & !($q << 1) & !($q >> 1) != 0
                    && $p & (($p << 3) | ($p << 2) | ($p << 1) | ($p >> 1) | ($p >> 2) | ($p >> 3)) != 0
            }};
        }

        self.black_pattern_available = calculate_available_pattern!(self.black_stones, self.white_stones);
        self.white_pattern_available = calculate_available_pattern!(self.white_stones, self.black_stones);
    }

}

#[derive(Copy, Clone)]
pub struct Slices {
    pub horizontal_slices: [Slice; pos::U_BOARD_WIDTH],
    pub vertical_slices: [Slice; pos::U_BOARD_WIDTH],
    pub ascending_slices: [Slice; DIAGONAL_SLICE_AMOUNT],
    pub descending_slices: [Slice; DIAGONAL_SLICE_AMOUNT],
}

impl Default for Slices {

    fn default() -> Self {
        Slices::EMPTY
    }

}

impl Slices {

    pub const EMPTY: Self = {
        let mut horizontal_slices = [Slice::PLACEHOLDER; pos::U_BOARD_WIDTH];
        let mut vertical_slices = [Slice::PLACEHOLDER; pos::U_BOARD_WIDTH];
        let mut ascending_slices = [Slice::PLACEHOLDER; DIAGONAL_SLICE_AMOUNT];
        let mut descending_slices = [Slice::PLACEHOLDER; DIAGONAL_SLICE_AMOUNT];

        let mut idx = 0;
        while idx < pos::U_BOARD_WIDTH {
            horizontal_slices[idx] = Slice::empty(pos::BOARD_WIDTH, idx as u8, 0);
            vertical_slices[idx] = Slice::empty(pos::BOARD_WIDTH, 0, idx as u8);

            idx += 1;
        }

        let mut idx = 0;
        while idx < DIAGONAL_SLICE_AMOUNT {
            let seq_num = idx as isize + 5 - pos::I_BOARD_WIDTH;

            ascending_slices[idx] = Slice::empty(
                (seq_num.abs() - pos::I_BOARD_WIDTH).unsigned_abs() as u8,
                max!(0, -seq_num) as u8,
                max!(0, seq_num) as u8
            );
            descending_slices[idx] = Slice::empty(
                (seq_num.abs() - pos::I_BOARD_WIDTH).unsigned_abs() as u8,
                pos::BOARD_WIDTH - 1 - max!(0, -seq_num) as u8,
                max!(0, seq_num) as u8
            );

            idx += 1;
        }

        Self {
            horizontal_slices,
            vertical_slices,
            ascending_slices,
            descending_slices,
        }
    };

    pub fn set_mut(&mut self, color: Color, pos: Pos) {
        self.horizontal_slices[pos.row_usize()].set_mut(color, pos.col());
        self.vertical_slices[pos.col_usize()].set_mut(color, pos.row());

        if let Some(ascending_slice) = Self::ascending_slice_idx(pos).map(|idx|
            &mut self.ascending_slices[idx]
        ) {
            ascending_slice.set_mut(color, pos.col() - ascending_slice.start_col)
        }

        if let Some(descending_slice) = Self::descending_slice_idx(pos).map(|idx|
            &mut self.descending_slices[idx]
        ) {
            descending_slice.set_mut(color, pos.col() - descending_slice.start_col)
        }
    }

    pub fn access_slice_unchecked(&self, direction: Direction, pos: Pos) -> &Slice {
        match direction {
            Direction::Horizontal => &self.horizontal_slices[pos.row_usize()],
            Direction::Vertical => &self.vertical_slices[pos.col_usize()],
            Direction::Ascending => &self.ascending_slices[Self::calculate_ascending_slice_idx(pos) as usize],
            Direction::Descending => &self.descending_slices[Self::calculate_descending_slice_idx(pos) as usize],
        }
    }
    
    fn calculate_ascending_slice_idx(pos: Pos) -> isize {
        // y = x + b, b = y - x (reversed row sequence)
        I_DIAGONAL_SLICE_AMOUNT - (pos.row() as isize - pos.col() as isize + pos::I_BOARD_WIDTH - 4)
    }

    pub fn ascending_slice_idx(pos: Pos) -> Option<usize> {
        let idx = Self::calculate_ascending_slice_idx(pos);
        (0 .. I_DIAGONAL_SLICE_AMOUNT).contains(&idx)
            .then_some(idx as usize)
    }
    
    fn calculate_descending_slice_idx(pos: Pos) -> isize {
        // y = -x + 15 + b, b = y + x - 15
        pos.row() as isize + pos.col() as isize - 4
    }

    pub fn descending_slice_idx(pos: Pos) -> Option<usize> {
        let idx = Self::calculate_descending_slice_idx(pos);
        (0 .. I_DIAGONAL_SLICE_AMOUNT).contains(&idx)
            .then_some(idx as usize)
    }

}
