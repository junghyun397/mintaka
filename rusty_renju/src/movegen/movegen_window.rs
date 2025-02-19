use crate::bitfield::Bitfield;
use crate::const_for;
use crate::notation::pos;
use crate::notation::pos::Pos;

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

    pub fn expand_window(&mut self, pos: Pos) {
        let row = pos.row();
        let col = pos.col();
        let max_bound = pos::U8_BOARD_SIZE - 1;

        self.start_row = self.start_row.min(row.saturating_sub(MOVEGEN_WINDOW_MARGIN));
        self.start_col = self.start_col.min(col.saturating_sub(MOVEGEN_WINDOW_MARGIN));
        self.end_row = self.end_row.max((row + MOVEGEN_WINDOW_MARGIN).min(max_bound));
        self.end_col = self.end_col.max((col + MOVEGEN_WINDOW_MARGIN).min(max_bound));
    }

    pub fn imprint_window(&mut self, pos: Pos) {
        self.movegen_field |= MOVEGEN_WINDOW_MASK_LUT[pos.idx_usize()];
    }

}

impl Into<Bitfield> for MovegenWindow {

    fn into(self) -> Bitfield {
        todo!()
    }

}

const MOVEGEN_WINDOW_MARGIN: u8 = 3;

const MOVEGEN_WINDOW_MASK_LUT: [Bitfield; pos::BOARD_SIZE] = build_movegen_window_mask_lut();

const fn build_movegen_window_mask_lut() -> [Bitfield; pos::BOARD_SIZE] {
    let line_4 = 0b1110111;
    let line_3 = 0b0111110;
    let line_2 = 0b0111110;
    let line_1 = 0b1001001;
    let lines = [line_1, line_2, line_3, line_4, line_3, line_2, line_1];

    let mut lut = [Bitfield::ZERO_FILLED; pos::BOARD_SIZE];

    const_for!(row in 0, pos::U_BOARD_WIDTH; {
        const_for!(col in 0, pos::U_BOARD_WIDTH; {
        })
    });

    lut
}
