use crate::game_state::GameState;
use crate::movegen::move_list::MoveList;
use crate::movegen::score_table::probe_score_table_lut;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::Score;
use rusty_renju::utils::platform;
use rusty_renju::{cartesian_to_index, chebyshev_distance, index_to_col, index_to_row, pattern};
use std::simd::cmp::SimdPartialEq;
use std::simd::Simd;

pub const VCF_MAX_MOVES: usize = 31;

#[derive(Debug, Copy, Clone)]
pub struct VcfMovesUnchecked {
    pub moves: [Pos; VCF_MAX_MOVES],
    pub top: u8,
}

impl VcfMovesUnchecked {

    pub fn unit(pos: Pos) -> Self {
        Self {
            moves: {
                const EMPTY_MOVES: [Pos; VCF_MAX_MOVES] = [MaybePos::INVALID_POS; VCF_MAX_MOVES];

                let mut new_moves = EMPTY_MOVES;
                new_moves[0] = pos;
                new_moves
            },
            top: 1,
        }
    }

    pub fn sort_moves(&mut self, ref_pos: Pos) {
        let ref_row = ref_pos.row() as i16;
        let ref_col = ref_pos.col() as i16;

        self.moves[..self.top as usize].sort_by_key(|&pos|
            chebyshev_distance!(ref_row, ref_col, pos.row() as i16, pos.col() as i16)
        );
    }

    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

}

fn score_move(state: &GameState, pos: Pos) -> Score {
    let neighborhood_score = state.move_scores.scores[pos.idx_usize()] as f32;
    let score =
        probe_score_table_lut::<{ Color::Black }>(
            state.board.patterns.field.get::<{ Color::Black }>()[pos.idx_usize()]
        ) as f32
            + probe_score_table_lut::<{ Color::White }>(
            state.board.patterns.field.get::<{ Color::White }>()[pos.idx_usize()]
        ) as f32;

    let distance = (state.history.avg_distance_to_recent_moves(pos).max(8) + 4) as f32;

    (((score + neighborhood_score) / distance) * 16.0) as Score
}

pub fn generate_vcf_moves(board: &Board, distance_window: isize, recent_move: Pos) -> VcfMovesUnchecked {
    let mut vcf_moves = [MaybePos::INVALID_POS; VCF_MAX_MOVES];
    let mut vcf_moves_top = 0;

    let field_ptr = board.patterns.field.access(board.player_color).as_ptr() as *const u32;

    let four_mask = Simd::splat(pattern::UNIT_ANY_FOUR_MASK);
    let zero_mask = Simd::splat(0);

    let recent_move_row = recent_move.row_usize();
    let recent_move_col = recent_move.col_usize();

    let begin_idx = {
        let begin_row = recent_move_row.saturating_sub(distance_window as usize);
        let begin_col = recent_move_col.saturating_sub(distance_window as usize);

        let begin_idx = cartesian_to_index!(begin_row, begin_col);
        begin_idx - begin_idx % platform::U32_LANE_N
    };

    let end_idx = {
        let end_row = (recent_move_row + distance_window as usize).max(pos::U_BOARD_WIDTH);
        let end_col = (recent_move_col + distance_window as usize).max(pos::U_BOARD_WIDTH);

        let end_idx = cartesian_to_index!(end_row, end_col);
        (end_idx + platform::U32_LANE_N).min(pos::BOARD_BOUND) - end_idx % platform::U32_LANE_N
    };

    for start_idx in (begin_idx .. end_idx).step_by(platform::U32_LANE_N) {
        let mut vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(field_ptr.add(start_idx), platform::U32_LANE_N) }
        );

        vector &= four_mask;
        let mut bitmask = vector
            .simd_ne(zero_mask)
            .to_bitmask();

        while bitmask != 0 {
            let lane_position = bitmask.trailing_zeros() as usize;
            bitmask &= bitmask - 1;

            let pos_idx = start_idx + lane_position;
            let distance = chebyshev_distance!(
                recent_move_row as isize, recent_move_col as isize,
                index_to_row!(pos_idx) as isize, index_to_col!(pos_idx) as isize
            );

            if distance > distance_window {
                continue;
            }

            vcf_moves[vcf_moves_top] = Pos::from_index(pos_idx as u8);
            vcf_moves_top += 1;
        }
    }

    if board.patterns.field.access(board.player_color)[pos::BOARD_BOUND].has_any_four() {
        vcf_moves[vcf_moves_top] = Pos::from_index(pos::U8_BOARD_BOUND);
        vcf_moves_top += 1;
    }

    VcfMovesUnchecked { moves: vcf_moves, top: vcf_moves_top as u8 }
}

pub fn generate_neighbors_moves(state: &GameState, moves: &mut MoveList) {
    let mut field = !state.board.hot_field & state.movegen_window.movegen_field;

    if state.board.player_color == Color::Black {
        field &= !state.board.patterns.forbidden_field;
    }

    for pos in field.iter_hot_pos() {
        moves.push(pos, score_move(state, pos));
    }
}

pub fn generate_defend_open_four_moves(state: &GameState, moves: &mut MoveList) {
    let player_ptr = state.board.patterns.field.access(state.board.player_color).as_ptr() as *const u32;
    let opponent_ptr = state.board.patterns.field.access(state.board.opponent_color()).as_ptr() as *const u32;

    let zero_mask = Simd::splat(0);
    let four_mask = Simd::splat(pattern::UNIT_ANY_FOUR_MASK);
    let close_three_mask = Simd::splat(pattern::UNIT_CLOSE_THREE_MASK);

    for start_idx in (0..pos::BOARD_BOUND).step_by(platform::U32_LANE_N) {
        let mut player_vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(player_ptr.add(start_idx), platform::U32_LANE_N) }
        );

        let mut opponent_vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(opponent_ptr.add(start_idx), platform::U32_LANE_N) }
        );

        player_vector &= four_mask;
        opponent_vector &= close_three_mask;

        let mut bitmask = (player_vector.simd_ne(zero_mask) | opponent_vector.simd_ne(zero_mask))
            .to_bitmask();

        while bitmask != 0 {
            let lane_position = bitmask.trailing_zeros() as usize;
            bitmask &= bitmask - 1;

            let pos = Pos::from_index((start_idx + lane_position) as u8);
            moves.push(pos, score_move(state, pos));
        }
    }

    if state.board.patterns.field.access(state.board.player_color)[pos::BOARD_BOUND].has_any_four()
        || state.board.patterns.field.access(state.board.opponent_color())[pos::BOARD_BOUND].has_close_three()
    {
        const POS: Pos = Pos::from_index(pos::U8_BOARD_BOUND);
        moves.push(POS, score_move(state, POS));
    }
}
