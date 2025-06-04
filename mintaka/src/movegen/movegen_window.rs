use rusty_renju::bitfield::Bitfield;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::{cartesian_to_index, const_for, max, min};

#[derive(Debug, Copy, Clone)]
pub struct MovegenWindow {
    pub start_row: u8,
    pub start_col: u8,
    pub end_row: u8,
    pub end_col: u8,
    pub movegen_field: Bitfield,
}

impl Default for MovegenWindow {

    fn default() -> Self {
        Self::DEFAULT
    }

}

const MOVEGEN_WINDOW_MARGIN: u8 = 3;

const MOVEGEN_IMPRINT_MASK_LUT: [Bitfield; pos::BOARD_SIZE] = build_movegen_imprint_mask_lut();

impl MovegenWindow {

    pub const DEFAULT: MovegenWindow = {
        let mut movegen_field = Bitfield::ZERO_FILLED;
        movegen_field.set_mut(pos::CENTER);

        MovegenWindow {
            start_row: pos::CENTER_ROW_COL,
            start_col: pos::CENTER_ROW_COL,
            end_row: pos::CENTER_ROW_COL,
            end_col: pos::CENTER_ROW_COL,
            movegen_field,
        }
    };

    fn expand_bounds_mut(&mut self, pos: Pos) {
        const MAX_BOUND: u8 = pos::BOARD_WIDTH - 1;

        let row = pos.row();
        let col = pos.col();

        self.start_row = self.start_row.min(row.saturating_sub(MOVEGEN_WINDOW_MARGIN));
        self.start_col = self.start_col.min(col.saturating_sub(MOVEGEN_WINDOW_MARGIN));
        self.end_row = self.end_row.max((row + MOVEGEN_WINDOW_MARGIN).min(MAX_BOUND));
        self.end_col = self.end_col.max((col + MOVEGEN_WINDOW_MARGIN).min(MAX_BOUND));
    }

    fn fill_bounds_mut(&mut self) {
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

    pub fn expand_window_mut(&mut self, pos: Pos) {
        self.expand_bounds_mut(pos);

        self.fill_bounds_mut();
    }

    pub fn batch_expand_window_mut(&mut self, moves: &[Pos]) {
        for &pos in moves {
            self.expand_bounds_mut(pos);
        }

        self.fill_bounds_mut();
    }

    pub fn imprint_window_mut(&mut self, pos: Pos) {
        self.movegen_field |= MOVEGEN_IMPRINT_MASK_LUT[pos.idx_usize()];
    }

    pub fn batch_imprint_window_mut(&mut self, moves: &[Pos]) {
        for pos in moves {
            self.movegen_field |= MOVEGEN_IMPRINT_MASK_LUT[pos.idx_usize()];
        }
    }

}

impl From<&Bitfield> for MovegenWindow {

    fn from(value: &Bitfield) -> Self {
        let mut acc = Self::default();

        for pos in value.iter_hot_pos() {
            acc.imprint_window_mut(pos);
        }

        acc
    }

}

const fn build_movegen_imprint_mask_lut() -> [Bitfield; pos::BOARD_SIZE] {
    let imprint_mask_pattern: [u16; 7] = [
        0b1001001,
        0b0111110,
        0b0111110,
        0b1110111,
        0b0111110,
        0b0111110,
        0b1001001,
    ];

    let mut lut = [Bitfield::ZERO_FILLED; pos::BOARD_SIZE];

    const_for!(row in 0, pos::I_BOARD_WIDTH; {
        const_for!(col in 0, pos::I_BOARD_WIDTH; {
            let row_begin = max!(row - 3, 0);
            let row_end = min!(row + 3, pos::I_BOARD_WIDTH - 1);
            let col_begin = max!(col - 3, 0);
            let col_end = min!(col + 3, pos::I_BOARD_WIDTH - 1);

            const_for!(row_offset in row_begin - row, row_end - row + 1; {
                const_for!(col_offset in col_begin - col, col_end - col + 1; {
                    if (imprint_mask_pattern[(row_offset + 3) as usize] >> (col_offset + 3)) & 0b1 == 0b1 {
                        let pos_idx = (row + row_offset) as usize * pos::U_BOARD_WIDTH + (col + col_offset) as usize;
                        lut[cartesian_to_index!(row, col) as usize].0[pos_idx / 8] |= 0b1 << (pos_idx % 8);
                    }
                });
            });
        })
    });

    lut
}
