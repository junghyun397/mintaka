use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::movegen::move_generator::generate_defend_open_four_moves;
use crate::movegen::move_list::{MoveEntry, MoveList};
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::score::{Score, Scores};

pub const KILLER_MOVE_SLOTS: usize = 2;

pub const TT_MOVE_SCORE: i16 = Score::INF as i16 - 500;
pub const KILLER_MOVE_SCORE: i16 = Score::INF as i16 - 1000;
pub const COUNTER_MOVE_SCORE: i16 = Score::INF as i16 - 2000;
pub const MAX_HISTORY_MOVE_SCORE: i16 = 10000;

#[derive(Eq, PartialEq)]
enum MoveStage {
    TT,
    Killer,
    Counter,
    GenerateAllMoves,
    AllMoves,
}

pub struct MovePicker {
    stage: MoveStage,
    forced: bool,
    moves_buffer: MoveList,
    tt_move: MaybePos,
    killer_moves: [MaybePos; KILLER_MOVE_SLOTS],
    counter_move: MaybePos,
    neural_scored: bool,
}

impl MovePicker {

    pub fn init_new(
        tt_move: MaybePos,
        killer_moves: [MaybePos; KILLER_MOVE_SLOTS],
        forced: bool
    ) -> Self {
        Self {
            stage: MoveStage::TT,
            forced,
            moves_buffer: MoveList::default(),
            tt_move,
            killer_moves,
            counter_move: MaybePos::NONE,
            neural_scored: false,
        }
    }

    pub fn next(
        &mut self,
        td: &mut ThreadData<impl ThreadType, impl Evaluator>,
        state: &GameState,
    ) -> Option<MoveEntry> {
        if self.stage == MoveStage::TT {
            self.stage = MoveStage::Killer;

            if self.tt_move.is_some() {
                return Some(MoveEntry {
                    pos: self.tt_move.unwrap(),
                    move_score: TT_MOVE_SCORE
                });
            }
        }

        if self.stage == MoveStage::Killer {
            loop {
                if self.killer_moves[0].is_none() {
                    self.stage = MoveStage::Counter;
                    break;
                }

                let killer_move = self.killer_moves[0];
                self.killer_moves[0] = self.killer_moves[1];
                self.killer_moves[1] = MaybePos::NONE;

                if killer_move != self.tt_move
                    && (!self.forced
                        || state.board.patterns.field[state.board.player_color][killer_move.unwrap().idx_usize()].has_close_three()
                    )
                {
                    return Some(MoveEntry {
                        pos: killer_move.unwrap(),
                        move_score: KILLER_MOVE_SCORE
                    });
                }
            }
        }

        if self.stage == MoveStage::Counter {
            self.stage = MoveStage::GenerateAllMoves;

            if !self.forced
                && state.history.len() > 1
                && let Some(prev_move) = state.history.recent_player_action().ok()
                && let maybe_counter_move = td.ht.counter[state.board.player_color][prev_move.idx_usize()]
                && maybe_counter_move.is_some()
                && maybe_counter_move != self.tt_move
                && maybe_counter_move != self.killer_moves[0]
                && maybe_counter_move != self.killer_moves[1]
            {
                self.counter_move = maybe_counter_move;

                return Some(MoveEntry {
                    pos: maybe_counter_move.unwrap(),
                    move_score: COUNTER_MOVE_SCORE
                });
            }
        }

        if self.stage == MoveStage::GenerateAllMoves {
            self.stage = MoveStage::AllMoves;

            if self.forced {
                generate_defend_open_four_moves(state, &mut self.moves_buffer);
            } else {
                self.score_and_push_all_moves(td, state);
            }
        }

        if self.stage == MoveStage::AllMoves {
            while let Some(next_move) = self.moves_buffer.consume_best() {
                if let maybe_next_move = next_move.pos.into() && (
                    maybe_next_move == self.tt_move
                        || maybe_next_move == self.killer_moves[0]
                        || maybe_next_move == self.killer_moves[1]
                        || maybe_next_move == self.counter_move
                ) {
                    continue;
                }

                return Some(next_move);
            }
        }

        None
    }

    fn score_and_push_all_moves(
        &mut self,
        td: &mut ThreadData<impl ThreadType, impl Evaluator>,
        state: &GameState
    ) {
        let policy_buffer = td.evaluator.eval_policy(state);

        self.neural_scored = false;

        let field = state.board.legal_field() & state.movegen_window.movegen_field;

        let player_pattern = state.board.patterns.field[state.board.player_color];

        for idx in field.iter_hot_idx() {
            let player_pattern = player_pattern[idx];

            let history_score = if player_pattern.has_threes() {
                td.ht.three[state.board.player_color][idx]
            } else if player_pattern.has_any_four() {
                td.ht.four[state.board.player_color][idx]
            } else {
                td.ht.quiet[state.board.player_color][idx]
            }.clamp(-MAX_HISTORY_MOVE_SCORE, MAX_HISTORY_MOVE_SCORE);

            self.moves_buffer.push(Pos::from_index(idx as u8), policy_buffer[idx] + history_score);
        }
    }

}
