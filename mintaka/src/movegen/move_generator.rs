use crate::movegen::move_list::MoveList;
use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::pattern;
use rusty_renju::utils::platform;
use std::simd::cmp::SimdPartialEq;
use std::simd::Simd;

#[derive(Debug, Copy, Clone)]
pub struct VcfMoves {
    pub moves: [Pos; 31],
    pub len: u8,
}

pub fn sort_moves(recent_move: Pos, moves: &mut [Pos]) {
    moves.sort_by(|&a, &b| {
        recent_move.distance(a).cmp(&recent_move.distance(b))
    });
}

pub fn generate_vcf_moves(board: &Board, color: Color, distance_window: u8, recent_move: Pos) -> VcfMoves {
    let mut vcf_moves = [MaybePos::NONE.unwrap(); 31];
    let mut vcf_moves_top = 0;

    let field_ptr = board.patterns.field.access(color).as_ptr() as *const u32;

    for start_idx in (0 .. pos::BOARD_BOUND).step_by(platform::U32_LANE_N) {
        let mut vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(field_ptr.add(start_idx), platform::U32_LANE_N) }
        );

        vector &= Simd::splat(pattern::UNIT_ANY_FOUR_MASK);
        let mut bitmask = vector
            .simd_ne(Simd::splat(0))
            .to_bitmask();

        while bitmask != 0 {
            let lane_position = bitmask.trailing_zeros() as usize;
            bitmask &= bitmask - 1;

            let pos = Pos::from_index((start_idx + lane_position) as u8);
            if recent_move.distance(pos) <= distance_window {
                vcf_moves[vcf_moves_top] = pos;
                vcf_moves_top += 1;
            }
        }
    }

    if board.patterns.field.access(color)[pos::BOARD_BOUND].has_any_four() {
        vcf_moves[vcf_moves_top] = Pos::from_index(pos::U8_BOARD_BOUND);
        vcf_moves_top += 1;
    }

    sort_moves(recent_move, &mut vcf_moves[..vcf_moves_top]);

    VcfMoves { moves: vcf_moves, len: vcf_moves_top as u8 }
}

pub fn generate_neighbors_moves(board: &Board, movegen_window: &MovegenWindow, moves: &mut MoveList) {
    for pos in (board.hot_field ^ movegen_window.movegen_field).iter_hot_pos() {
        moves.push(pos, 0); // TODO: distance score
    }
}

pub fn generate_defend_three_moves(board: &Board, moves: &mut MoveList) {
    match board.player_color {
        Color::Black => generate_defend_three_moves_impl::<{ Color::Black }>(board, moves),
        Color::White => generate_defend_three_moves_impl::<{ Color::White }>(board, moves)
    }
}

fn generate_defend_three_moves_impl<const C: Color>(board: &Board, moves: &mut MoveList) {
    let player_ptr = board.patterns.field.player_unit::<C>().as_ptr() as *const u32;
    let opponent_ptr = board.patterns.field.opponent_unit::<C>().as_ptr() as *const u32;

    for start_idx in (0..pos::BOARD_BOUND).step_by(platform::U32_LANE_N) {
        let mut player_vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(player_ptr.add(start_idx), platform::U32_LANE_N) }
        );

        let mut opponent_vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(opponent_ptr.add(start_idx), platform::U32_LANE_N) }
        );

        player_vector &= Simd::splat(pattern::UNIT_ANY_FOUR_MASK);
        opponent_vector &= Simd::splat(pattern::UNIT_CLOSE_THREE_MASK | pattern::UNIT_OPEN_FOUR_MASK);

        let mut bitmask = (
            player_vector.simd_ne(Simd::splat(0)) | opponent_vector.simd_ne(Simd::splat(0))
        ).to_bitmask();

        while bitmask != 0 {
            let lane_position = bitmask.trailing_zeros() as usize;
            bitmask &= bitmask - 1;

            let idx = start_idx + lane_position;
            let player_pattern = board.patterns.field.player_unit::<C>()[idx];

            if C == Color::Black && player_pattern.is_forbidden() {
                continue;
            }

            moves.push(Pos::from_index(idx as u8), 0); // TODO: distance score
        }
    }

    let player_pattern = board.patterns.field.player_unit::<C>()[pos::BOARD_BOUND];
    let opponent_pattern = board.patterns.field.opponent_unit::<C>()[pos::BOARD_BOUND];

    if (!player_pattern.is_empty() || !opponent_pattern.is_empty())
        && (player_pattern.has_any_four() || opponent_pattern.has_close_three())
        && !(C == Color::Black && player_pattern.is_forbidden())
    {
        moves.push(Pos::from_index(pos::U8_BOARD_BOUND), 0); // TODO: distance score
    }
}

pub fn is_open_four_available(board: &Board) -> bool {
    match board.player_color {
        Color::Black => is_open_four_available_black(board),
        Color::White => is_open_four_available_white(board),
    }
}

fn is_open_four_available_black(board: &Board) -> bool {
    let field_ptr = board.patterns.field.black.as_ptr() as *const u32;

    for start_idx in (0 ..pos::BOARD_BOUND).step_by(platform::U32_LANE_N) {
        let mut vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(field_ptr.add(start_idx), platform::U32_LANE_N) }
        );

        vector &= Simd::splat(pattern::UNIT_OPEN_FOUR_MASK);
        let mut bitmask = vector
            .simd_ne(Simd::splat(0))
            .to_bitmask();

        while bitmask != 0 {
            let lane_position = bitmask.trailing_zeros() as usize;
            bitmask &= bitmask - 1;

            if !board.patterns.field.black[start_idx + lane_position].is_forbidden() {
                return true;
            }
        }
    }

    false
}

fn is_open_four_available_white(board: &Board) -> bool {
    let field_ptr = board.patterns.field.white.as_ptr() as *const u32;

    for start_idx in (0 ..pos::BOARD_BOUND).step_by(platform::U32_LANE_N) {
        let mut vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(field_ptr.add(start_idx), platform::U32_LANE_N) }
        );

        vector &= Simd::splat(pattern::UNIT_OPEN_FOUR_MASK);
        if vector.simd_ne(Simd::splat(0)).any() {
            return true;
        }
    }

    board.patterns.field.white[pos::BOARD_BOUND].has_open_four()
}
