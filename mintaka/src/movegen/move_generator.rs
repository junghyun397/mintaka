use rusty_renju::bitfield::Bitfield;
use crate::search_endgame::{EndgameMovesUnchecked, ENDGAME_MAX_MOVES};
use rusty_renju::board::Board;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::score::{Score, Scores};
use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::movegen::move_list::MoveList;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;

pub const TT_MOVE_SCORE: i16 = Score::INF as i16 - 300;
pub const DIRECT_RESPONSE_SCORE: i16 = Score::INF as i16 - 500;
pub const KILLER_MOVE_SCORE: i16 = Score::INF as i16 - 1000;
pub const COUNTER_MOVE_BONUS: i16 = 100;

pub fn generate_endgame_moves<const VCT: bool>(board: &Board, distance_window: u8, recent_move: Pos) -> EndgameMovesUnchecked {
    let mut vcf_moves = [MaybePos::NONE; ENDGAME_MAX_MOVES];
    let mut vcf_moves_top = 0;

    let mut field = board.patterns.indexes[board.player_color].closed_fours;

    if VCT {
        field |= board.patterns.indexes[board.player_color].open_threes;
    }

    for pos in field.iter_hot_pos() {
        if pos.distance(recent_move) > distance_window {
            continue;
        }

        vcf_moves[vcf_moves_top] = pos.into();
        vcf_moves_top += 1;
    }

    EndgameMovesUnchecked { moves: vcf_moves, top: vcf_moves_top as u8 }
}

pub fn generate_threat_direct_response(
    buffer: &mut MoveList,
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    state: &GameState,
    field: &Bitfield,
) {
    for pos in field.iter_hot_pos() {
        let mut score = DIRECT_RESPONSE_SCORE;

        // threat score
        if state.board.patterns.field[state.board.player_color][pos.idx_usize()].has_any_threat() {
            score += 100;
        }

        buffer.push(pos, score, false, None);
    }
}

pub fn generate_extend_four_response(
    buffer: &mut MoveList,
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    state: &GameState,
) {
    let maybe_last_pos = state.history.last_action_or_none();

    let counter_move = counter_move_from(td, state)
        .filter(|pos| state.board.patterns.field[state.board.player_color][pos.idx_usize()].has_closed_four());

    for pos in state.board.patterns.indexes[state.board.player_color].closed_fours.iter_hot_pos() {
        let mut score = 0;

        // distance score
        if let Some(last_pos) = maybe_last_pos.ok() {
            score += (15 - pos.distance(last_pos)) as i16;
        }

        // counter-move score
        if let Some(counter_move) = counter_move && counter_move == pos {
            score += COUNTER_MOVE_BONUS;
        }

        // history score
        score += td.ht.four[state.board.player_color][pos.idx_usize()] / 128;

        buffer.push(pos, score, false, None);
    }
}

pub fn generate_all_moves(
    buffer: &mut MoveList,
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    state: &GameState,
) {
    let policy_buffer = td.evaluator.eval_policy(state);

    let field = state.board.legal_field(state.board.player_color) & state.movegen_window.movegen_field;
    let player_pattern = state.board.patterns.field[state.board.player_color];

    let counter_move = counter_move_from(td, state);

    for pos in field.iter_hot_pos() {
        let idx = pos.idx_usize();
        let player_pattern = player_pattern[idx];

        // policy score
        let mut score = policy_buffer[idx];

        // counter-move score
        if let Some(counter_move) = counter_move && pos == counter_move {
            score += COUNTER_MOVE_BONUS;
        }

        // history score
        let history_score;
        if player_pattern.has_open_three() {
            history_score = td.ht.three[state.board.player_color][idx];

            score += history_score / 256;
        } else if player_pattern.has_any_four() {
            history_score = td.ht.four[state.board.player_color][idx];

            score += history_score / 256;
        } else {
            history_score = td.ht.quiet[state.board.player_color][idx];

            score += history_score / 512;
        };

        buffer.push(Pos::from_index(idx as u8), score, false, Some(history_score));
    }
}

fn counter_move_from(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    state: &GameState,
) -> Option<Pos> {
    state.history.last_action()
        .and_then(|action| action.ok())
        .and_then(|last_pos|
            td.ht.counter[state.board.player_color][last_pos.idx_usize()].ok()
        )
}
