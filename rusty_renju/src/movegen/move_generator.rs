use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::pos::Pos;
use smallvec::{smallvec, SmallVec};

pub type Moves = SmallVec<[Pos; 32]>;

pub fn sort_moves(recent_move: Pos, moves: &mut Moves) {
    fn distance_to_recent_move(recent_move: Pos, pos: Pos) -> u8 {
        let row_diff = (recent_move.row() as i16 - pos.row() as i16).unsigned_abs();
        let col_diff = (recent_move.col() as i16 - pos.col() as i16).unsigned_abs();

        row_diff.max(col_diff) as u8
    }

    moves.sort_by(|a, b| {
        distance_to_recent_move(recent_move, *a).cmp(&distance_to_recent_move(recent_move, *b))
    });
}

pub fn generate_moves<const C: Color>(board: &Board) -> Moves {
    let mut defend_five_pos = Pos::INVALID;
    let mut on_defend_five_position = false;

    let mut defend_threat_moves = smallvec![];
    let mut on_defend_threat_position = false;

    for (idx, pattern) in board.patterns.field.iter().enumerate() {
        if pattern.is_empty() {
            continue;
        }

        let player_unit = pattern.player_unit::<C>();

        if player_unit.has_five() {
            return smallvec![Pos::from_index(idx as u8)];
        }

        let opponent_unit = pattern.opponent_unit::<C>();

        if opponent_unit.has_five() && !pattern.is_forbidden() {
            defend_five_pos = Pos::from_index(idx as u8);
            on_defend_five_position = true;
            continue;
        }

        if !on_defend_five_position
            && (player_unit.has_any_four() || player_unit.has_close_three())
            && !(C == Color::Black && pattern.is_forbidden())
        {
            defend_threat_moves.push(Pos::from_index(idx as u8));
            continue;
        }

        if match C {
            Color::Black => !on_defend_five_position
                && (opponent_unit.has_open_four() || opponent_unit.has_fours())
                && !pattern.is_forbidden(),
            Color::White => !on_defend_five_position
                && opponent_unit.has_open_four()
                && !pattern.is_forbidden()
        } {
            defend_threat_moves.push(Pos::from_index(idx as u8));
            on_defend_threat_position = true;
        }
    }

    if on_defend_five_position {
        smallvec![defend_five_pos]
    } else if on_defend_threat_position {
        defend_threat_moves
    } else {
        todo!()
    }
}

pub fn generate_neighborhood_moves(board: &Board) -> Bitfield {
    todo!()
}

pub fn generate_opening_moves(board: &Board, ) -> Bitfield {
    todo!()
}
