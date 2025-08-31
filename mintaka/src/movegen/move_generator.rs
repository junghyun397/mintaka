use crate::endgame::accumulator::{EndgameMovesUnchecked, ENDGAME_MAX_MOVES};
use crate::game_state::GameState;
use crate::movegen::move_list::MoveList;
use rusty_renju::board::Board;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::Score;
use rusty_renju::utils::platform;
use rusty_renju::{cartesian_to_index, chebyshev_distance, index_to_col, index_to_row, pattern};
use std::simd::cmp::SimdPartialEq;
use std::simd::Simd;

fn score_move(state: &GameState, pos: Pos) -> Score {
    let distance = (state.history.avg_distance_to_recent_actions(pos).max(8) + 4) as f32;

    16 - distance as Score
}

pub fn generate_vcf_moves(board: &Board, distance_window: isize, recent_move: Pos) -> EndgameMovesUnchecked {
    let mut vcf_moves = [MaybePos::NONE; ENDGAME_MAX_MOVES];
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
        begin_idx - begin_idx % platform::U32_WIDE_LANE_N
    };

    let end_idx = {
        let end_row = (recent_move_row + distance_window as usize).max(pos::U_BOARD_WIDTH);
        let end_col = (recent_move_col + distance_window as usize).max(pos::U_BOARD_WIDTH);

        let end_idx = cartesian_to_index!(end_row, end_col);
        (end_idx + platform::U32_WIDE_LANE_N).min(pattern::PATTERN_SIZE) - end_idx % platform::U32_WIDE_LANE_N
    };

    for start_idx in (begin_idx .. end_idx).step_by(platform::U32_WIDE_LANE_N) {
        let mut vector = Simd::<u32, { platform::U32_WIDE_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(field_ptr.add(start_idx), platform::U32_WIDE_LANE_N) }
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

            vcf_moves[vcf_moves_top] = Pos::from_index(pos_idx as u8).into();
            vcf_moves_top += 1;
        }
    }

    EndgameMovesUnchecked { moves: vcf_moves, top: vcf_moves_top as u8 }
}

pub fn generate_defend_open_four_moves(state: &GameState, moves: &mut MoveList) {
    let player_ptr = state.board.patterns.field.access(state.board.player_color).as_ptr() as *const u32;
    let opponent_ptr = state.board.patterns.field.access(!state.board.player_color).as_ptr() as *const u32;

    let zero_mask = Simd::splat(0);
    let four_mask = Simd::splat(pattern::UNIT_ANY_FOUR_MASK);
    let close_three_mask = Simd::splat(pattern::UNIT_CLOSE_THREE_MASK);

    for start_idx in (0 .. pattern::PATTERN_SIZE).step_by(platform::U32_WIDE_LANE_N) {
        let mut player_vector = Simd::<u32, { platform::U32_WIDE_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(player_ptr.add(start_idx), platform::U32_WIDE_LANE_N) }
        );

        let mut opponent_vector = Simd::<u32, { platform::U32_WIDE_LANE_N }>::from_slice(
            unsafe { std::slice::from_raw_parts(opponent_ptr.add(start_idx), platform::U32_WIDE_LANE_N) }
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
}
