use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::notation::color::Color;
use crate::opening::opening_agent::OpeningStage;
use crate::pattern::Pattern;
use ethnum::{u256, uint};

pub fn generate_moves(board: &Board) -> Bitfield {
    todo!()
}

pub fn generate_neighborhood_moves(board: &Board) -> Bitfield {
    todo!()
}

pub fn generate_defend_four_moves(board: &Board) -> Bitfield {
    match board.player_color {
        Color::Black => generate_defend_move_by_condition(board, |(_, pattern)|
            pattern.white_unit.has_close_three()
                && !pattern.is_forbidden()
        ),
        Color::White => generate_defend_move_by_condition(board, |(_, pattern)|
            pattern.black_unit.has_close_three()
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
