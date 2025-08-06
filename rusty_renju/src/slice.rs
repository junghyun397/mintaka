use crate::bitfield::Bitfield;
use crate::notation::color::{Color, ColorContainer};
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::{assert_struct_sizes, const_for, max, slice_pattern};

pub const DIAGONAL_SLICE_AMOUNT: usize = pos::U_BOARD_WIDTH * 2 - 4 - 4 - 1;
const I_DIAGONAL_SLICE_AMOUNT: isize = DIAGONAL_SLICE_AMOUNT as isize;
pub const TOTAL_SLICE_AMOUNT: usize = pos::U_BOARD_WIDTH * 2 + DIAGONAL_SLICE_AMOUNT * 2;
const DIAGONAL_BOARD_PADDING: isize = 5 - pos::I_BOARD_WIDTH;

#[derive(Debug, Copy, Clone)]
#[repr(align(16))]
pub struct Slice {
    pub idx: u8,
    pub length: u8,
    pub start_row: u8,
    pub start_col: u8,
    pub start_pos: Pos,
    pub stones: ColorContainer<u16>,
    pub pattern_bitmap: ColorContainer<u16>,
}

assert_struct_sizes!(Slice, size=16, align=16);

impl Slice {

    pub const PLACEHOLDER: Self = unsafe { std::mem::zeroed() };

    pub const fn empty(idx: u8, length: u8, start_row: u8, start_col: u8) -> Self {
        Slice {
            idx,
            length,
            start_row,
            start_col,
            start_pos: Pos::from_cartesian(start_row, start_col),
            stones: ColorContainer::new(0, 0),
            pattern_bitmap: ColorContainer::new(0, 0),
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
        *self.stones.access_mut(color) |= 0b1 << idx;
    }

    pub fn unset_mut(&mut self, color: Color, idx: u8) {
        *self.stones.access_mut(color) &= !(0b1 << idx);
    }

    pub fn stones<const C: Color>(&self) -> u16 {
        self.stones.get::<C>()
    }

    pub fn stones_reversed_color<const C: Color>(&self) -> u16 {
        self.stones.get_reversed::<C>()
    }

    pub fn is_empty(&self, idx: u8) -> bool {
        let mask = 0b1 << idx;
        (self.stones.black | self.stones.white) & mask == 0
    }

    pub fn stone_kind(&self, idx: u8) -> Option<Color> {
        let mask = 0b1 << idx;

        if self.stones.black & mask == mask {
            Some(Color::Black)
        } else if self.stones.white & mask == mask {
            Some(Color::White)
        } else {
            None
        }
    }

    pub fn calculate_slice_idx(&self, direction: Direction, pos: Pos) -> usize {
        match direction {
            Direction::Vertical => pos.row_usize(),
            Direction::Horizontal => pos.col_usize(),
            _ => pos.col_usize() - self.start_col as usize
        }
    }

    pub fn has_potential_pattern<const C: Color>(&self) -> bool {
        // filter . . O . . . .
        // filter O X . . O X .
        // filter O . . . O . .
        #[inline(always)]
        fn is_pattern_available(p: u16, q: u16) -> bool {
            p.count_ones() > 1
                && p & !(q << 1) & !(q >> 1) != 0
                && p & ((p << 3) | (p << 2) | (p << 1) | (p >> 1) | (p >> 2) | (p >> 3)) != 0
        }

        match C {
            Color::Black => is_pattern_available(self.stones.black, self.stones.white),
            Color::White => is_pattern_available(self.stones.white, self.stones.black),
        }
    }

    pub fn winner(&self) -> Option<Color> {
        if slice_pattern::contains_five_in_a_row(self.stones.black) {
            Some(Color::Black)
        } else if slice_pattern::contains_five_in_a_row(self.stones.white) {
            Some(Color::White)
        } else {
            None
        }
    }

    // TODO: add pre-check and optimization; 7%.
    #[inline(always)]
    pub fn evaluate_score<const C: Color>(&self) -> i16 {
        let blocks = {
            let raw = self.stones.get_reversed::<C>() as u32
                | self.pattern_bitmap.get::<C>() as u32;

            (raw << 1) | 0b1
        };

        let stones = (self.stones.get::<C>() as u32) << 1;

        let p1 = blocks & (blocks >> 2);
        let p2 = blocks & (blocks >> 3);
        let p3 = blocks & (blocks >> 4);
        let p4 = blocks & (blocks >> 5);

        let filled = (p1 << 1)
            | (p2 << 1) | (p2 << 2)
            | (p3 << 1) | (p3 << 2) | (p3 << 3)
            | (p4 << 1) | (p4 << 2) | (p4 << 3) | (p4 << 4);

        (!filled & stones).count_ones() as i16
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

        const_for!(idx in 0, pos::U_BOARD_WIDTH; {
            horizontal_slices[idx] = Slice::empty(idx as u8, pos::BOARD_WIDTH, idx as u8, 0);
            vertical_slices[idx] = Slice::empty(idx as u8, pos::BOARD_WIDTH, 0, idx as u8);
        });

        const_for!(idx in 0, DIAGONAL_SLICE_AMOUNT; {
            let seq_num = idx as isize + DIAGONAL_BOARD_PADDING;
            let len = (seq_num.abs() - pos::I_BOARD_WIDTH).unsigned_abs() as u8;
            let start_offset = max!(0, -seq_num) as u8;
            let end_offset = max!(0, seq_num) as u8;

            ascending_slices[idx] = Slice::empty(
                idx as u8,
                len,
                start_offset,
                end_offset,
            );
            descending_slices[idx] = Slice::empty(
                idx as u8,
                len,
                pos::BOARD_WIDTH - 1 - start_offset,
                end_offset
            );
        });

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

    pub fn bitfield(&self) -> ColorContainer<Bitfield> {
        self.horizontal_slices.iter()
            .enumerate()
            .fold(
                ColorContainer::new(Bitfield::default(), Bitfield::default()),
                  |mut bitfield_container, (row_idx, slice)| {
                      for col_idx in 0..pos::BOARD_WIDTH {
                          if let Some(color) = slice.stone_kind(col_idx) {
                              bitfield_container
                                  .access_mut(color)
                                  .set_mut(Pos::from_cartesian(row_idx as u8, col_idx));
                          }
                      }

                      bitfield_container
                }
            )
    }

}
