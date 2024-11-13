use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::pos::BOARD_SIZE;
use crate::opening::opening_agent::OpeningStage;
use crate::pattern::Pattern;
use crate::{index_to_col, index_to_row};
use ethnum::{u256, uint};

const NEIGHBORHOOD_MOVES_LUT: [Bitfield; BOARD_SIZE] = build_neighborhood_moves_lut();

pub fn generate_moves(board: &Board) -> Bitfield {
    todo!()
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
        .fold(u256::MIN, |mut acc, (idx, _)| {
            acc |= uint!("0b1") << idx;
            acc
        })
}

pub fn generate_opening_moves(board: &Board, agent: OpeningStage) -> Bitfield {
    todo!()
}

const fn build_neighborhood_moves_lut() -> [Bitfield; BOARD_SIZE] {
    let mut lut = [u256::MIN; BOARD_SIZE];

    let mut idx = 0;
    while idx < BOARD_SIZE {
        let row = index_to_row!(idx);
        let col = index_to_col!(idx);
        idx += 1;
    }

    lut
}
