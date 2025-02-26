use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::movegen::movegen_window::MovegenWindow;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::pattern;
use crate::pattern::Pattern;
use smallvec::SmallVec;
use std::simd::cmp::SimdPartialEq;
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

pub fn generate_moves(board: &Board, movegen_window: &MovegenWindow) -> Moves {
    match board.player_color {
        Color::Black => generate_moves_with_color::<{ Color::Black }>(board, movegen_window),
        Color::White => generate_moves_with_color::<{ Color::White }>(board, movegen_window),
    }
}

fn generate_moves_with_color<const C: Color>(board: &Board, movegen_window: &MovegenWindow) -> Moves {
    // if is_open_four_available(board, !C) {
    //     generate_defend_three_moves::<C>(board)
    // } else {
    //     SmallVec::from_iter((!board.hot_field & movegen_window.movegen_field).iter_hot_pos())
    // }
    todo!()
}

pub fn generate_neighborhood_moves(board: &Board, movegen_window: &MovegenWindow) -> Moves {
    SmallVec::from_iter((!board.hot_field & movegen_window.movegen_field).iter_hot_pos())
}

pub fn generate_defend_three_moves<const C: Color>(board: &Board) -> Moves {
    let mut defend_threat_moves = SmallVec::new();

    for idx in 0 .. pos::BOARD_SIZE {
        let player_pattern = board.patterns.field.player_unit::<C>()[idx];
        let opponent_pattern = board.patterns.field.opponent_unit::<C>()[idx];

        if player_pattern.is_empty() && opponent_pattern.is_empty() {
            continue;
        }

        if (player_pattern.has_any_four() || opponent_pattern.has_close_three())
            && !(C == Color::Black && player_pattern.is_forbidden())
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

pub fn open_four_positions(board: &Board, color: Color) -> SmallVec<[Pos; 4]> {
    let mut acc = SmallVec::new();
    let mask = u32x16::splat(pattern::UNIT_OPEN_FOUR_MASK);
    let zeros = u32x16::splat(0);

    let patterns = unsafe {
        std::mem::transmute::<[Pattern; pos::BOARD_SIZE], [u32; pos::BOARD_SIZE]>(*board.patterns.field.access(color))
    };

    for vector_idx in 0 .. pos::BOARD_SIZE / 16 {
        let vector = u32x16::from_slice(&patterns[vector_idx * 16..vector_idx * 16 + 16]);

        let masked = unsafe {
            // core::intrinsics::simd::simd_select_bitmask(mask, vector, mask)
            vector
        };

        let mut bitmask = masked
            .simd_ne(zeros)
            .to_bitmask();

        while bitmask != 0 {
            let lane_position = bitmask.trailing_zeros() as usize;
            acc.push(Pos::from_index((vector_idx * 16 + lane_position) as u8));
            bitmask &= bitmask - 1;
        }
    }

    acc
}

fn is_open_four_available_white(board: &Board, color: Color) -> bool {
    let patterns = unsafe {
        std::mem::transmute::<[Pattern; pos::BOARD_SIZE], [u32; pos::BOARD_SIZE]>(*board.patterns.field.access(color))
    };

    let mut acc = u32x16::from_slice(&patterns[0 .. 16]);

    // total 12 iterations
    for chunk in patterns.chunks_exact(16).skip(1) {
        acc |= u32x16::from_slice(chunk);
    }

    // (15 * 15) mod 16 = 1
    let merged_pattern = unsafe { std::mem::transmute::<u32, Pattern>(acc.reduce_or()) };
    merged_pattern.has_open_four() | board.patterns.field.access(color)[pos::BOARD_BOUND].has_open_four()
}
