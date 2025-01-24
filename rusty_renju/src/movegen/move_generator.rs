use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::Pos;
use std::simd::u64x64;

const MOVE_SET_TABLE: [Bitfield; pos::BOARD_SIZE] = {
    let mut move_set_table = [Bitfield::ZERO_FILLED; pos::BOARD_SIZE];

    let mut idx = 0;
    while idx < pos::BOARD_SIZE {
        let base_pos = Pos::from_index(idx as u8);

        let mut offset_row = -2;
        while offset_row <= 2 {
            let mut offset_col = -2;
            while offset_col <= 2 {
                if let Some(pos) = base_pos.offset(offset_row, offset_col) {
                    move_set_table[idx].set_mut(pos);
                }
                offset_col += 1;
            }
            offset_row += 1;
        }
        idx += 1;
    }

    move_set_table
};

pub fn threat_available(pattern_vector: u64x64) -> bool {
    false
}

pub fn generate_moves<const C: Color>(board: &Board) -> Bitfield {
    // if open_four or five is available...

    let mut defend_five_field = Bitfield::default();
    let mut on_defend_five_position = false;

    let mut defend_open_four_field = Bitfield::default();
    let mut on_defend_open_four_position = false;

    for (idx, pattern) in board.patterns.field.iter().enumerate() {
        if pattern.is_empty() {
            continue;
        }

        let player_unit = pattern.player_unit::<C>();

        if player_unit.has_five() {
            let mut result = Bitfield::default();
            result.set_mut(Pos::from_index(idx as u8));
            return result;
        }

        let opponent_unit = pattern.opponent_unit::<C>();

        if opponent_unit.has_five() && !pattern.is_forbidden() {
            defend_five_field.set_mut(Pos::from_index(idx as u8));
            on_defend_five_position = true;
        }

        if !on_defend_five_position
            && player_unit.has_any_four()
            && !(C == Color::Black && pattern.is_forbidden())
        {
            defend_open_four_field.set_mut(Pos::from_index(idx as u8));
        }

        match C {
            Color::Black => {
                if !on_defend_five_position
                    && (opponent_unit.has_open_four() || opponent_unit.has_fours())
                    && !pattern.is_forbidden()
                {
                    defend_open_four_field.set_mut(Pos::from_index(idx as u8));
                    on_defend_open_four_position = true;
                }
            }
            Color::White => {
                if !on_defend_five_position
                    && opponent_unit.has_open_four()
                    && !pattern.is_forbidden()
                {
                    defend_open_four_field.set_mut(Pos::from_index(idx as u8));
                    on_defend_open_four_position = true;
                }
            }
        }
    }

    if on_defend_five_position {
        defend_five_field
    } else if on_defend_open_four_position {
        defend_open_four_field
    } else {
        !board.hot_field
    }
}

pub fn generate_neighborhood_moves(board: &Board) -> Bitfield {
    todo!()
}

pub fn generate_opening_moves(board: &Board, ) -> Bitfield {
    todo!()
}
