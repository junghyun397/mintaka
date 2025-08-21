use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::movegen::move_generator::generate_defend_open_four_moves;
use crate::movegen::move_list::{MoveEntry, MoveList};
use crate::search_frame::KILLER_MOVE_SLOTS;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::{Score, Scores};

pub const TT_MOVE_SCORE: Score = Score::INF - 500;
pub const KILLER_MOVE_SCORE: Score = Score::INF - 1000;
pub const HISTORY_MOVE_SCORE: Score = Score::INF - 2000;

#[derive(Eq, PartialEq)]
enum MoveStage {
    TT,
    Killer,
    DefendFour,
    Neighbor,
}

pub struct MovePicker {
    stage: MoveStage,
    moves_buffer: MoveList,
    tt_move: MaybePos,
    killer_moves: [MaybePos; KILLER_MOVE_SLOTS],
}

impl MovePicker {

    pub fn new(
        tt_move: MaybePos,
        killer_moves: [MaybePos; KILLER_MOVE_SLOTS],
    ) -> Self {
        Self {
            stage: MoveStage::TT,
            moves_buffer: MoveList::default(),
            tt_move,
            killer_moves,
        }
    }

    pub fn next(
        &mut self,
        td: &ThreadData<impl ThreadType, impl Evaluator>,
        state: &GameState,
    ) -> Option<MoveEntry> {
        loop {
            match self.stage {
                MoveStage::TT => {
                    self.stage = MoveStage::Killer;

                    if self.tt_move.is_some() {
                        return Some(MoveEntry {
                            pos: self.tt_move.unwrap(),
                            score: TT_MOVE_SCORE
                        });
                    }
                },
                MoveStage::Killer => {
                    if self.killer_moves[0].is_some() {
                        let killer_move = self.killer_moves[0].unwrap();

                        self.killer_moves[0] = self.killer_moves[1];
                        self.killer_moves[1] = MaybePos::NONE;

                        return Some(MoveEntry {
                            pos: killer_move,
                            score: KILLER_MOVE_SCORE
                        });
                    }

                    if Self::opponent_has_open_four(state) {
                        generate_defend_open_four_moves(state, &mut self.moves_buffer);
                        self.stage = MoveStage::DefendFour;
                    } else {
                        let mut field = !state.board.hot_field & state.movegen_window.movegen_field;

                        if state.board.player_color == Color::Black {
                            field &= !state.board.patterns.forbidden_field;
                        }

                        let policy_buffer = td.evaluator.eval_policy(state);

                        for idx in field.iter_hot_idx() {
                            self.moves_buffer.push(Pos::from_index(idx as u8), policy_buffer[idx]);
                        }

                        self.stage = MoveStage::Neighbor;
                    }
                },
                MoveStage::DefendFour | MoveStage::Neighbor => {
                    return self.moves_buffer.consume_best();
                },
            }
        }
    }

    fn opponent_has_open_four(state: &GameState) -> bool {
        let total_fours = match !state.board.player_color {
            Color::Black => {
                let mut total_fours = state.board.patterns.counts.global.black.open_fours as u32;

                total_fours -= state.board.patterns.forbidden_field.iter_hot_idx()
                    .map(|idx| state.board.patterns.field.black[idx].count_open_fours())
                    .sum::<u32>();

                total_fours
            },
            Color::White => state.board.patterns.counts.global.white.open_fours as u32
        };

        total_fours != 0
    }

}
