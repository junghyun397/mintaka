use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::movegen::movegen_window::MovegenWindow;
use crate::notation::color::Color;
use crate::notation::pos::Pos;
use crate::opening::opening_agent::OpeningAgent;
use smallvec::{smallvec, SmallVec};

pub type Moves = SmallVec<[Pos; 32]>;

pub fn generate_moves<const C: Color>(board: &Board, movegen_window: MovegenWindow) -> Moves {
    if is_threat_available(board) {
        generate_threat_moves::<C>(board)
    } else {
        generate_neighborhood_moves(board, movegen_window)
    }
}

pub fn generate_neighborhood_moves(board: &Board, movegen_window: MovegenWindow) -> Moves {
    SmallVec::from_iter((!board.hot_field & movegen_window.movegen_field).iter_hot_pos())
}

pub fn generate_threat_moves<const C: Color>(board: &Board) -> Moves {
    let mut defend_threat_moves = SmallVec::new();
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

        if (player_unit.has_any_four() || player_unit.has_close_three())
            && !(C == Color::Black && pattern.is_forbidden())
        {
            defend_threat_moves.push(Pos::from_index(idx as u8));
            continue;
        }

        if match C {
            Color::Black => (opponent_unit.has_open_four() || opponent_unit.has_fours())
                && !pattern.is_forbidden(),
            Color::White => opponent_unit.has_open_four()
                && !pattern.is_forbidden()
        } {
            defend_threat_moves.push(Pos::from_index(idx as u8));
            on_defend_threat_position = true;
        }
    }

    defend_threat_moves
}

fn sort_moves(recent_move: Pos, moves: &mut Moves) {
    fn distance_to_recent_move(recent_move: Pos, pos: Pos) -> u8 {
        let row_diff = (recent_move.row() as i16 - pos.row() as i16).unsigned_abs();
        let col_diff = (recent_move.col() as i16 - pos.col() as i16).unsigned_abs();

        row_diff.max(col_diff) as u8
    }

    moves.sort_by(|a, b| {
        distance_to_recent_move(recent_move, *a).cmp(&distance_to_recent_move(recent_move, *b))
    });
}

pub fn generate_opening_moves(board: &Board, opening_agent: &dyn OpeningAgent) -> Bitfield {
    todo!()
}

fn is_threat_available(board: &Board) -> bool {
    false
}
