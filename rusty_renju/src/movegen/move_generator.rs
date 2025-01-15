use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::pos::Pos;
use crate::opening::opening_agent::OpeningStage;
use crate::pattern::Pattern;

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
        .fold(Bitfield::ZERO_FILLED, |mut acc, (idx, _)| {
            acc.set(Pos::from_index(idx as u8));
            acc
        })
}

pub fn generate_opening_moves(board: &Board, agent: OpeningStage) -> Bitfield {
    todo!()
}
