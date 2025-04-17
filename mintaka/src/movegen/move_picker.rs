use crate::game_state::GameState;
use crate::movegen::move_generator::{generate_defend_three_moves, generate_neighbors_moves};
use crate::movegen::move_list::MoveList;
use crate::search_frame::KILLER_MOVE_SLOTS;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::{Score, Scores};

pub const TT_MOVE_SCORE: Score = Score::INF - 500;
pub const KILLER_MOVE_SCORE: Score = Score::INF - 1000;
pub const COUNTER_MOVE_SCORE: Score = Score::INF - 2000;

#[derive(Eq, PartialEq)]
enum MoveStage {
    TT,
    Killer,
    Counter,
    DefendFour,
    Neighbor,
    Done
}

pub struct MovePicker {
    stage: MoveStage,
    moves: MoveList,
    tt_move: MaybePos,
    killer_moves: [MaybePos; KILLER_MOVE_SLOTS],
    counter_move: MaybePos,
}

impl MovePicker {

    pub fn new(
        tt_move: MaybePos,
        killer_moves: [MaybePos; KILLER_MOVE_SLOTS],
        counter_move: MaybePos,
    ) -> Self {
        Self {
            stage: MoveStage::TT,
            moves: MoveList::default(),
            tt_move,
            killer_moves,
            counter_move,
        }
    }

    pub fn next(
        &mut self,
        state: &GameState,
    ) -> Option<(Pos, Score)> {
        match self.stage {
            MoveStage::TT => {
                self.stage = MoveStage::Killer;

                if self.tt_move.is_some() {
                    return Some((self.tt_move.unwrap(), TT_MOVE_SCORE));
                }

                self.next(state)
            },
            MoveStage::Killer => {
                if let Some(killer_move) = self.killer_moves.iter_mut()
                    .find(|action| action.is_some())
                {
                        let pos = killer_move.unwrap();
                        *killer_move = MaybePos::NONE;

                        return Some((pos, KILLER_MOVE_SCORE));
                }

                self.stage = MoveStage::Counter;

                self.next(state)
            },
            MoveStage::Counter => {
                if self.counter_move.is_some() {
                    let counter_move = self.counter_move.unwrap();

                    self.counter_move = MaybePos::NONE;

                    return Some((counter_move, COUNTER_MOVE_SCORE));
                }

                if state.board.patterns.score_table.slice_pattern_count.access(state.board.opponent_color()).open_fours != 0 {
                    generate_defend_three_moves(state, &mut self.moves);
                    self.stage = MoveStage::DefendFour;
                } else {
                    generate_neighbors_moves(state, &mut self.moves);
                    self.stage = MoveStage::Neighbor;
                }

                self.next(state)
            },
            MoveStage::DefendFour | MoveStage::Neighbor => {
                let next_move = self.moves.consume_best();

                if next_move.is_none() {
                    self.stage = MoveStage::Done;
                }

                next_move
            },
            MoveStage::Done => None
        }
    }

}
