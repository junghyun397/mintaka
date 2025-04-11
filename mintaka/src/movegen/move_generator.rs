use crate::game_state::GameState;
use crate::movegen::move_list::MoveList;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::pattern;
use rusty_renju::utils::platform;
use std::simd::cmp::SimdPartialEq;
use std::simd::Simd;

#[derive(Debug, Copy, Clone)]
pub struct VcfMovesUnchecked {
    pub moves: [Pos; 31],
    pub top: u8,
}

impl VcfMovesUnchecked {

    pub fn sort_moves(&mut self, ref_pos: Pos) {
        let ref_row = ref_pos.row();
        let ref_col = ref_pos.col();

        self.moves[..self.top as usize].sort_by_key(|&pos|
            pos::chebyshev_distance(ref_row, ref_col, pos.row(), pos.col())
        );
    }

}

fn score_move(state: &GameState, pos: Pos) -> i16 {
    let distance = 5u8.saturating_sub(state.history.multi_distance(pos));
    state.move_scores.scores[pos.idx_usize()] as i16 * distance as i16
}

pub fn generate_vcf_moves(board: &Board, color: Color, distance_window: u8, recent_four: Pos) -> VcfMovesUnchecked {
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
            if recent_four.distance(pos) <= distance_window {
                vcf_moves[vcf_moves_top] = pos;
                vcf_moves_top += 1;
            }
        }
    }

    if board.patterns.field.access(color)[pos::BOARD_BOUND].has_any_four() {
        vcf_moves[vcf_moves_top] = Pos::from_index(pos::U8_BOARD_BOUND);
        vcf_moves_top += 1;
    }

    VcfMovesUnchecked { moves: vcf_moves, top: vcf_moves_top as u8 }
}

pub fn generate_neighbors_moves(state: &GameState, moves: &mut MoveList) {

    for pos in (state.board.hot_field ^ state.movegen_window.movegen_field).iter_hot_pos() {
        moves.push(pos, score_move(state, pos));
    }
}

pub fn generate_defend_three_moves(state: &GameState, moves: &mut MoveList) {
    match state.board.player_color {
        Color::Black => generate_defend_three_moves_impl::<{ Color::Black }>(state, moves),
        Color::White => generate_defend_three_moves_impl::<{ Color::White }>(state, moves)
    }
}

fn generate_defend_three_moves_impl<const C: Color>(state: &GameState, moves: &mut MoveList) {
    let player_ptr = state.board.patterns.field.player_unit::<C>().as_ptr() as *const u32;
    let opponent_ptr = state.board.patterns.field.opponent_unit::<C>().as_ptr() as *const u32;

    for start_idx in (0..pos::BOARD_BOUND).step_by(platform::U32_LANE_N) {
        let mut player_vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(player_ptr.add(start_idx), platform::U32_LANE_N) }
        );

        let mut opponent_vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(opponent_ptr.add(start_idx), platform::U32_LANE_N) }
        );

        player_vector &= Simd::splat(pattern::UNIT_ANY_FOUR_MASK);
        opponent_vector &= Simd::splat(pattern::UNIT_CLOSE_THREE_MASK);

        let mut bitmask = (
            player_vector.simd_ne(Simd::splat(0)) | opponent_vector.simd_ne(Simd::splat(0))
        ).to_bitmask();

        while bitmask != 0 {
            let lane_position = bitmask.trailing_zeros() as usize;
            bitmask &= bitmask - 1;

            let idx = start_idx + lane_position;
            let player_pattern = state.board.patterns.field.player_unit::<C>()[idx];

            if C == Color::Black && player_pattern.is_forbidden() {
                continue;
            }

            let pos = Pos::from_index(idx as u8);
            moves.push(pos, score_move(state, pos));
        }
    }

    let player_pattern = state.board.patterns.field.player_unit::<C>()[pos::BOARD_BOUND];
    let opponent_pattern = state.board.patterns.field.opponent_unit::<C>()[pos::BOARD_BOUND];

    if (!player_pattern.is_empty() || !opponent_pattern.is_empty())
        && (player_pattern.has_any_four() || opponent_pattern.has_close_three())
        && !(C == Color::Black && player_pattern.is_forbidden())
    {
        const POS: Pos = Pos::from_index(pos::U8_BOARD_BOUND);
        moves.push(POS, score_move(state, POS));
    }
}

pub fn is_open_four_available(board: &Board) -> bool {
    match board.player_color {
        Color::Black => is_open_four_available_white(board),
        Color::White => is_open_four_available_black(board),
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
