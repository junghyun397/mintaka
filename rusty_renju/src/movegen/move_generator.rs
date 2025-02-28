use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::movegen::movegen_window::MovegenWindow;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::pattern;
use crate::pattern::Pattern;
use crate::utils::platform;
use smallvec::SmallVec;
use std::simd::cmp::SimdPartialEq;
use std::simd::Simd;

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
        Color::Black => {
            if is_open_four_available_white(board) {
                return generate_defend_three_moves::<{ Color::Black }>(board);
            }
        },
        Color::White => {
            if is_open_four_available_black(board) {
                return generate_defend_three_moves::<{ Color::White }>(board);
            }
        }
    }

    generate_neighborhood_moves(board, movegen_window)
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

fn generate_neighborhood_moves(board: &Board, movegen_window: &MovegenWindow) -> Moves {
    SmallVec::from_iter((board.hot_field ^ movegen_window.movegen_field).iter_hot_pos())
}

fn generate_defend_three_moves<const C: Color>(board: &Board) -> Moves {
    let mut defend_threat_moves = SmallVec::new();

    let player_patterns = unsafe {
        std::mem::transmute::<[Pattern; pos::BOARD_SIZE], [u32; pos::BOARD_SIZE]>(
            *board.patterns.field.player_unit::<C>()
        )
    };

    let opponent_patterns = unsafe {
        std::mem::transmute::<[Pattern; pos::BOARD_SIZE], [u32; pos::BOARD_SIZE]>(
            *board.patterns.field.opponent_unit::<C>()
        )
    };

    for start_idx in (0..pos::BOARD_BOUND).step_by(platform::U32_LANE_N) {
        let mut player_vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            &player_patterns[start_idx..start_idx + 16]
        );
        let mut opponent_vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            &opponent_patterns[start_idx..start_idx + 16]
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
            let opponent_pattern = board.patterns.field.opponent_unit::<C>()[idx];

            if (player_pattern.has_any_four() || opponent_pattern.has_close_three())
                && (C == Color::White || !player_pattern.is_forbidden())
            {
                defend_threat_moves.push(Pos::from_index(idx as u8));
            }
        }
    }

    let player_pattern = board.patterns.field.player_unit::<C>()[pos::BOARD_BOUND];
    let opponent_pattern = board.patterns.field.opponent_unit::<C>()[pos::BOARD_BOUND];

    if (player_pattern.has_any_four() || opponent_pattern.has_close_three())
        && !(C == Color::Black && player_pattern.is_forbidden())
    {
        defend_threat_moves.push(Pos::from_index(pos::U8_BOARD_BOUND));
    }

    defend_threat_moves
}

fn is_open_four_available_black(board: &Board) -> bool {
    let patterns = unsafe {
        std::mem::transmute::<[Pattern; pos::BOARD_SIZE], [u32; pos::BOARD_SIZE]>(board.patterns.field.black)
    };

    for start_idx in (0 ..pos::BOARD_BOUND).step_by(platform::U32_LANE_N) {
        let mut vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            &patterns[start_idx .. start_idx + platform::U32_LANE_N]
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
    let patterns = unsafe {
        std::mem::transmute::<[Pattern; pos::BOARD_SIZE], [u32; pos::BOARD_SIZE]>(board.patterns.field.white)
    };

    for start_idx in (0 ..pos::BOARD_BOUND).step_by(platform::U32_LANE_N) {
        let mut vector = Simd::<u32, { platform::U32_LANE_N }>::from_slice(
            &patterns[start_idx .. start_idx + platform::U32_LANE_N]
        );

        vector &= Simd::splat(pattern::UNIT_OPEN_FOUR_MASK);
        if vector.simd_ne(Simd::splat(0)).any() {
            return true;
        }
    }

    board.patterns.field.white[pos::BOARD_BOUND].has_open_four()
}
