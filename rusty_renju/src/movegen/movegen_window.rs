use crate::bitfield::Bitfield;
use crate::notation::pos;
use crate::notation::pos::Pos;

#[derive(Debug, Copy, Clone)]
pub struct MovegenWindow {
    pub start_row: u8,
    pub start_col: u8,
    pub end_row: u8,
    pub end_col: u8,
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
    };

    pub fn set_mut(&mut self, pos: Pos) {
        let row = pos.row();
        let col = pos.col();
        let margin = 3;
        let max_bound = pos::U8_BOARD_SIZE - 1;

        self.start_row = self.start_row.min(row.saturating_sub(margin));
        self.start_col = self.start_col.min(col.saturating_sub(margin));
        self.end_row = self.end_row.max((row + margin).min(max_bound));
        self.end_col = self.end_col.max((col + margin).min(max_bound));
    }

}

impl Into<Bitfield> for MovegenWindow {

    fn into(self) -> Bitfield {
        todo!()
    }

}
