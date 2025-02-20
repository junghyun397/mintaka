use crate::bitfield::Bitfield;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::{const_for, max, min};

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
        Self::EMPTY
    }

}

impl MovegenWindow {

    pub const EMPTY: MovegenWindow = MovegenWindow {
        start_row: pos::CENTER_ROW_COL,
        start_col: pos::CENTER_ROW_COL,
        end_row: pos::CENTER_ROW_COL,
        end_col: pos::CENTER_ROW_COL,
        movegen_field: Bitfield::ZERO_FILLED,
    };

    pub fn expand_window_mut(&mut self, pos: Pos) {
        let row = pos.row();
        let col = pos.col();
        let max_bound = pos::U8_BOARD_SIZE - 1;

        self.start_row = self.start_row.min(row.saturating_sub(MOVEGEN_WINDOW_MARGIN));
        self.start_col = self.start_col.min(col.saturating_sub(MOVEGEN_WINDOW_MARGIN));
        self.end_row = self.end_row.max((row + MOVEGEN_WINDOW_MARGIN).min(max_bound));
        self.end_col = self.end_col.max((col + MOVEGEN_WINDOW_MARGIN).min(max_bound));
    }

    pub fn imprint_window_mut(&mut self, pos: Pos) {
        self.movegen_field |= MOVEGEN_WINDOW_MASK_LUT[pos.idx_usize()];
    }

}

const MOVEGEN_WINDOW_MARGIN: u8 = 3;

const MOVEGEN_WINDOW_MASK_LUT: [Bitfield; pos::BOARD_SIZE] = build_movegen_window_mask_lut();

const fn build_movegen_window_mask_lut() -> [Bitfield; pos::BOARD_SIZE] {
    let window_mask_pattern: [u16; 7] = [
        0b10010010_00000000,
        0b0111110_00000000,
        0b0111110_00000000,
        0b1110111_00000000,
        0b0111110_00000000,
        0b0111110_00000000,
        0b1001001_00000000,
    ];

    let mut lut = [Bitfield::ZERO_FILLED; pos::BOARD_SIZE];

    const_for!(row in 0, pos::U_BOARD_WIDTH; {
        const_for!(col in 0, pos::U_BOARD_WIDTH; {
            let col_shl = -min!(col as isize - 3, 0);
            let col_shr = min!(col + 3, pos::U_BOARD_WIDTH - 1) as isize;

            let row_begin = max!(row as isize - 3, 0);
            let row_end = min!(row + 3, pos::U_BOARD_WIDTH - 1) as isize;

            const_for!(row_offset in 0, row_end - row_begin; {
                let mut mask = window_mask_pattern[row_offset as usize];
                mask <<= col_shl;
                mask >>= col_shr;
                mask &= !0b1;

                let shift = (row_begin + row_offset) as usize * pos::U_BOARD_WIDTH + col;

                let bitfield_idx = shift / 8;
                let bitfield_shift = shift % 8;

                let bitfield_segments = [
                    (mask >> (8 + bitfield_shift)) as u8,
                    (mask >> bitfield_shift) as u8,
                    (mask << (8 - bitfield_shift)) as u8
                ];

                const_for!(segment_idx in 0, 3; {
                    lut[row * pos::U_BOARD_WIDTH + col].0[bitfield_idx + segment_idx] |= bitfield_segments[segment_idx];
                });
            });
        })
    });

    lut
}
