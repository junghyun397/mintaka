use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::opening::opening_agent::OpeningStage;
use crate::pattern::Pattern;

pub const MOVE_SET_TABLE: [Bitfield; pos::BOARD_SIZE] = {
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

pub fn generate_moves<const C: Color>(board: &Board) -> Bitfield {
    let mut defend_four_field = Bitfield::default();
    let mut on_four_threat = false;

    for (idx, pattern) in board.patterns.field.iter().enumerate() {
        let player_unit = pattern.player_unit::<C>();
        let opponent_unit = pattern.opponent_unit::<C>();
        if player_unit.has_five() || opponent_unit.has_five() {
            let mut result = Bitfield::default();
            result.set_mut(Pos::from_index(idx as u8));
            return result;
        }

        match C {
            Color::Black => {
                if (opponent_unit.has_open_four() || opponent_unit.has_fours()) && !pattern.is_forbidden() {
                    defend_four_field.set_mut(Pos::from_index(idx as u8));
                    on_four_threat = true;
                }
            }
            Color::White => {
                if opponent_unit.has_open_four() && !pattern.is_forbidden() {
                    defend_four_field.set_mut(Pos::from_index(idx as u8));
                    on_four_threat = true;
                }
            }
        }
    }

    if on_four_threat {
        defend_four_field
    } else {
        !board.hot_field
    }
}

pub fn generate_neighborhood_moves(board: &Board) -> Bitfield {
    todo!()
}

pub fn generate_defend_three_moves(board: &Board) -> Bitfield {
    match board.player_color {
        Color::Black => generate_defend_move_by_condition(board, |(_, pattern)|
            (pattern.black_unit.has_fours() || pattern.white_unit.has_close_three())
                && !pattern.is_forbidden()
        ),
        Color::White => generate_defend_move_by_condition(board, |(_, pattern)|
            pattern.white_unit.has_fours() || pattern.white_unit.has_close_three()
        )
    }
}

pub fn generate_defend_four_moves(board: &Board) -> Bitfield {
    match board.player_color {
        Color::Black => generate_defend_move_by_condition(board, |(_, pattern)|
            pattern.white_unit.has_five()
                && !pattern.is_forbidden()
        ),
        Color::White => generate_defend_move_by_condition(board, |(_, pattern)|
            pattern.black_unit.has_five()
        )
    }
}

pub fn generate_defend_move_by_condition<T>(board: &Board, cond: T) -> Bitfield
where T : Fn(&(usize, &Pattern)) -> bool {
    board.patterns.field.iter()
        .enumerate()
        .filter(cond)
        .fold(Bitfield::ZERO_FILLED, |mut acc, (idx, _)| {
            acc.set_mut(Pos::from_index(idx as u8));
            acc
        })
}

pub fn generate_opening_moves(board: &Board, agent: OpeningStage) -> Bitfield {
    todo!()
}
