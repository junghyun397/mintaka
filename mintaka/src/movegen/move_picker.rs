use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::movegen::move_list::{MoveEntry, MoveList};
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::bitfield::Bitfield;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::score::{Score, Scores};
use crate::movegen::move_generator::generate_defend_open_four_moves;

pub const KILLER_MOVE_SLOTS: usize = 2;

pub const TT_MOVE_SCORE: i16 = Score::INF as i16 - 500;
pub const KILLER_MOVE_SCORE: i16 = Score::INF as i16 - 1000;
pub const COUNTER_MOVE_SCORE: i16 = Score::INF as i16 - 2000;
pub const MAX_HISTORY_MOVE_SCORE: i16 = 10000;

#[derive(Copy, Clone, Eq, PartialEq)]
enum MoveStage {
    TT,
    Killer,
    Counter,
    Generate(MoveKind),
    Moves(MoveKind),
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum MoveKind {
    All,
    DefendOpenFour,
    ExtendFour,
}

pub struct MovePicker {
    stage: MoveStage,
    forced_defense_three: bool,
    moves_buffer: MoveList,
    fours_buffer: Bitfield,
    tt_move: MaybePos,
    killer_moves: [MaybePos; KILLER_MOVE_SLOTS],
    occupied_moves: Bitfield,
}

impl MovePicker {

    pub fn init_new(
        tt_move: MaybePos,
        killer_moves: [MaybePos; KILLER_MOVE_SLOTS],
        forced_defense_three: bool
    ) -> Self {
        Self {
            stage: MoveStage::TT,
            forced_defense_three,
            moves_buffer: MoveList::EMPTY,
            fours_buffer: Bitfield::ZERO_FILLED,
            tt_move,
            killer_moves,
            occupied_moves: Bitfield::ZERO_FILLED,
        }
    }

    pub fn next(
        &mut self,
        td: &mut ThreadData<impl ThreadType, impl Evaluator>,
        state: &GameState,
    ) -> Option<MoveEntry> {
        loop {
            match self.stage {
                MoveStage::TT => {
                    self.stage = MoveStage::Killer;

                    if let Some(tt_move) = self.tt_move.ok()
                        && self.is_forced_legal(state, tt_move)
                    {
                        self.occupied_moves.set(tt_move);

                        return Some(MoveEntry {
                            pos: tt_move,
                            move_score: TT_MOVE_SCORE
                        });
                    }
                }
                MoveStage::Killer => {
                    loop {
                        let Some(killer_move) = self.killer_moves[0].ok() else {
                            self.stage = MoveStage::Counter;
                            break;
                        };

                        self.killer_moves[0] = self.killer_moves[1];
                        self.killer_moves[1] = MaybePos::NONE;

                        if self.occupied_moves.is_cold(killer_move)
                            && self.is_forced_legal(state, killer_move)
                        {
                            self.occupied_moves.set(killer_move);

                            return Some(MoveEntry {
                                pos: killer_move,
                                move_score: KILLER_MOVE_SCORE
                            });
                        }
                    }
                }
                MoveStage::Counter => {
                    if self.forced_defense_three {
                        self.stage = MoveStage::Generate(MoveKind::DefendOpenFour);
                    } else {
                        self.stage = MoveStage::Generate(MoveKind::All);
                    }

                    if state.history.len() > 1
                        && let Some(prev_move) = state.history.last_action_unchecked().ok()
                        && let Some(counter_move) = td.ht.counter[state.board.player_color][prev_move.idx_usize()].ok()
                        && self.occupied_moves.is_cold(counter_move)
                        && self.is_forced_legal(state, counter_move)
                    {
                        self.occupied_moves.set(counter_move);

                        return Some(MoveEntry {
                            pos: counter_move,
                            move_score: COUNTER_MOVE_SCORE
                        });
                    }
                }
                MoveStage::Generate(kind) => {
                    match kind {
                        MoveKind::All =>
                            score_and_push_all_moves(&mut self.moves_buffer, td, state),
                        MoveKind::DefendOpenFour =>
                            generate_defend_open_four_moves(&mut self.moves_buffer, &mut self.fours_buffer, &state.board),
                        MoveKind::ExtendFour =>
                            score_and_push_extend_four_moves(&mut self.moves_buffer, td, state, &self.fours_buffer),
                    }

                    self.stage = MoveStage::Moves(kind);
                }
                MoveStage::Moves(kind) => {
                    while let Some(next_move) = self.moves_buffer.consume_best() {
                        if self.occupied_moves.is_hot(next_move.pos) {
                            continue;
                        }

                        self.occupied_moves.set(next_move.pos);

                        return Some(next_move);
                    }

                    match kind {
                        MoveKind::DefendOpenFour => self.stage = MoveStage::Generate(MoveKind::ExtendFour),
                        _ => return None,
                    }
                }
            }
        }
    }

    fn is_forced_legal(&self, state: &GameState, pos: Pos) -> bool {
        if !self.forced_defense_three {
            return true;
        }

        state.board.patterns.field[!state.board.player_color][pos.idx_usize()].has_close_three()
            || state.board.patterns.field[state.board.player_color][pos.idx_usize()].has_any_four()
    }

}

fn score_and_push_all_moves(
    buffer: &mut MoveList,
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    state: &GameState
) {
    let policy_buffer = td.evaluator.eval_policy(state);

    let field = state.board.legal_field(state.board.player_color) & state.movegen_window.movegen_field;

    let player_pattern = state.board.patterns.field[state.board.player_color];

    for idx in field.iter_hot_idx() {
        let player_pattern = player_pattern[idx];

        let history_score = if player_pattern.has_three() {
            td.ht.three[state.board.player_color][idx] / 64
        } else if player_pattern.has_any_four() {
            td.ht.four[state.board.player_color][idx] / 64
        } else {
            td.ht.quiet[state.board.player_color][idx] / 128
        };

        buffer.push(Pos::from_index(idx as u8), policy_buffer[idx] + history_score);
    }
}

fn score_and_push_extend_four_moves(
    buffer: &mut MoveList,
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    state: &GameState,
    fours_buffer: &Bitfield,
) {
    let policy_buffer = td.evaluator.eval_policy(state);

    for idx in fours_buffer.iter_hot_idx() {

        let history_score = td.ht.four[state.board.player_color][idx] / 64;

        buffer.push(Pos::from_index(idx as u8), policy_buffer[idx] + history_score);
    }
}
