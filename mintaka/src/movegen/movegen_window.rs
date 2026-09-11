use rusty_renju::bitfield::{Bitfield, build_imprint_mask_lut};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;

#[derive(Debug, Copy, Clone)]
pub struct MovegenWindow {
    start_row: u8,
    start_col: u8,
    end_row: u8,
    end_col: u8,
    pub movegen_field: Bitfield,
}

impl Default for MovegenWindow {
    fn default() -> Self {
        Self::DEFAULT
    }
}

const MOVEGEN_WINDOW_MARGIN: u8 = 3;

const MOVEGEN_IMPRINT_MASK_LUT: [Bitfield; pos::BOARD_SIZE] = build_imprint_mask_lut([
    0b1001001,
    0b0111110,
    0b0111110,
    0b1110111,
    0b0111110,
    0b0111110,
    0b1001001,
]);

impl MovegenWindow {

    pub const EMPTY: Self = Self {
        start_row: 0,
        start_col: 0,
        end_row: 0,
        end_col: 0,
        movegen_field: Bitfield::ZERO_FILLED,
    };

    pub const DEFAULT: Self = {
        let mut movegen_field = Bitfield::ZERO_FILLED;
        movegen_field.set(pos::CENTER);

        Self {
            start_row: pos::CENTER_ROW_COL,
            start_col: pos::CENTER_ROW_COL,
            end_row: pos::CENTER_ROW_COL,
            end_col: pos::CENTER_ROW_COL,
            movegen_field,
        }
    };

    pub const FULL: Self = Self {
        start_row: 0,
        start_col: 0,
        end_row: pos::BOARD_WIDTH,
        end_col: pos::BOARD_WIDTH,
        movegen_field: Bitfield::ONE_FILLED,
    };

    fn expand_bounds(&mut self, pos: Pos) {
        const MAX_BOUND: u8 = pos::BOARD_WIDTH - 1;

        let row = pos.row();
        let col = pos.col();

        self.start_row = self.start_row.min(row.saturating_sub(MOVEGEN_WINDOW_MARGIN));
        self.start_col = self.start_col.min(col.saturating_sub(MOVEGEN_WINDOW_MARGIN));
        self.end_row = self.end_row.max((row + MOVEGEN_WINDOW_MARGIN).min(MAX_BOUND));
        self.end_col = self.end_col.max((col + MOVEGEN_WINDOW_MARGIN).min(MAX_BOUND));
    }

    fn fill_bounds(&mut self) {
        for row in self.start_row ..= self.end_row {
            let row_idx = row as usize * pos::U_BOARD_WIDTH;
            let start_idx = row_idx + self.start_col as usize;
            let end_idx = row_idx + self.end_col as usize;

            let start_byte = start_idx / 8;
            let end_byte = end_idx / 8;
            let start_bit = start_idx % 8;
            let end_bit = end_idx % 8;

            if start_byte == end_byte {
                self.movegen_field.0[start_byte] |= (u8::MAX >> (7 - end_bit + start_bit)) << start_bit;
            } else {
                self.movegen_field.0[start_byte] |= u8::MAX << start_bit;
                self.movegen_field.0[end_byte]   |= u8::MAX >> (7 - end_bit);
            }
        }
    }

    pub fn expand_window(&mut self, pos: Pos) {
        self.expand_bounds(pos);

        self.fill_bounds();
    }

    pub fn imprint_window(&mut self, pos: Pos) {
        self.movegen_field |= MOVEGEN_IMPRINT_MASK_LUT[pos.idx_usize()];
    }

    pub fn batch_imprint_window(&mut self, moves: &[Pos]) {
        for pos in moves {
            self.movegen_field |= MOVEGEN_IMPRINT_MASK_LUT[pos.idx_usize()];
        }
    }

}

impl From<&Bitfield> for MovegenWindow {
    fn from(value: &Bitfield) -> Self {
        let mut acc = Self::default();

        for pos in value.iter_hot_pos() {
            acc.imprint_window(pos);
        }

        acc
    }
}
