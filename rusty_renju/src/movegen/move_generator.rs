use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::movegen::movegen_window::MovegenWindow;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::pattern::Pattern;
use smallvec::SmallVec;
use std::simd::num::SimdUint;
use std::simd::u32x16;

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
    if is_open_four_available::<C>(board) {
        generate_defend_threat_moves::<C>(board)
    } else {
        SmallVec::from_iter((!board.hot_field & movegen_window.movegen_field).iter_hot_pos())
    }
}

pub fn generate_neighborhood_moves(board: &Board, movegen_window: &MovegenWindow) -> Moves {
    SmallVec::from_iter((!board.hot_field & movegen_window.movegen_field).iter_hot_pos())
}

pub fn generate_defend_threat_moves<const C: Color>(board: &Board) -> Moves {
    let mut defend_threat_moves = SmallVec::new();

    for (idx, pattern) in board.patterns.field.player_unit::<C>().iter().enumerate() {
        if pattern.is_empty() {
            continue;
        }

        if (pattern.has_any_four() || pattern.has_close_three())
            && !(C == Color::Black && pattern.is_forbidden())
        {
            defend_threat_moves.push(Pos::from_index(idx as u8));
            continue;
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

pub fn is_open_four_available<const C: Color>(board: &Board) -> bool {
    match C {
        Color::Black => {
            todo!()
        }
        Color::White => {
            const THREAT_MASK: u32x16 = u32x16::from_array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

            let patterns = unsafe {
                std::mem::transmute::<[Pattern; pos::BOARD_SIZE], [u32; pos::BOARD_SIZE]>(board.patterns.field.white)
            };

            let mut acc = u32x16::from_slice(&patterns[0 .. 16]);

            // (15 * 15) mod 16 = 1
            for idx in (16 .. pos::BOARD_BOUND).step_by(16) {
                acc |= u32x16::from_slice(&patterns[idx .. idx + 16]);
            }

            let merged_pattern = unsafe { std::mem::transmute::<u32, Pattern>(acc.reduce_or()) };
            merged_pattern.has_open_four() | board.patterns.field.white[pos::BOARD_BOUND].has_open_four()
        }
    }
}
