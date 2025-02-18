use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::movegen::movegen_window::MovegenWindow;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::pattern::Pattern;
use smallvec::SmallVec;
use std::simd::num::SimdUint;
use std::simd::u64x8;

pub type Moves = SmallVec<[Pos; 64]>;

impl From<Moves> for Bitfield {

    fn from(moves: Moves) -> Self {
        let mut bitfield = Bitfield::ZERO_FILLED;

        for pos in moves {
            bitfield.set_mut(pos);
        }

        bitfield
    }

}

pub fn generate_moves<const C: Color>(board: &Board, movegen_window: &MovegenWindow) -> Moves {
    if is_threat_available(board) {
        generate_threat_moves::<C>(board)
    } else {
        SmallVec::from_iter((!board.hot_field & movegen_window.movegen_field).iter_hot_pos())
    }
}

pub fn generate_neighborhood_moves(board: &Board, movegen_window: &MovegenWindow) -> Moves {
    SmallVec::from_iter((!board.hot_field & movegen_window.movegen_field).iter_hot_pos())
}

fn generate_threat_moves<const C: Color>(board: &Board) -> Moves {
    let mut defend_threat_moves = SmallVec::new();

    for (idx, pattern) in board.patterns.field.iter().enumerate() {
        if pattern.is_empty() {
            continue;
        }

        let player_unit = pattern.player_unit::<C>();
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

pub fn is_threat_available(board: &Board) -> bool {
    let patterns = unsafe {
        &*(&board.patterns.field as *const [Pattern; pos::BOARD_SIZE] as *const [u64; pos::BOARD_SIZE])
    };

    let mut acc = u64x8::from_slice(&patterns[0 .. 8]);
    for idx in (8 .. pos::BOARD_SIZE / 8).step_by(8) {
        acc |= u64x8::from_slice(&patterns[idx .. idx + 8]);
    }

    const TAIL_BEGIN: usize = pos::BOARD_SIZE - pos::BOARD_SIZE % 8;
    let mut tail = [0; 8];
    tail[.. TAIL_BEGIN].copy_from_slice(&patterns[TAIL_BEGIN ..]);
    acc |= u64x8::from_array(tail);

    let merged_pattern = unsafe { std::mem::transmute::<u64, Pattern>(acc.reduce_or()) };
    merged_pattern.black.has_open_four() || merged_pattern.white.has_open_four()
}
