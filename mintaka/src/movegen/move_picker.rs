use crate::game_state::GameState;
use crate::movegen::move_generator::{generate_defend_open_four_moves, generate_neighbors_moves};
use crate::movegen::move_list::MoveList;
use crate::search_frame::KILLER_MOVE_SLOTS;
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
    Done
}

pub struct MovePicker {
    stage: MoveStage,
    moves: MoveList,
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
            moves: MoveList::default(),
            tt_move,
            killer_moves,
        }
    }

    pub fn next(
        &mut self,
        state: &GameState,
    ) -> Option<(Pos, Score)> {
        loop {
            match self.stage {
                MoveStage::TT => {
                    self.stage = MoveStage::Killer;

                    if self.tt_move.is_some() {
                        return Some((self.tt_move.unwrap(), TT_MOVE_SCORE));
                    }
                },
                MoveStage::Killer => {
                    if self.killer_moves[0].is_some() {
                        let killer_move = self.killer_moves[0].unwrap();

                        self.killer_moves[0] = self.killer_moves[1];
                        self.killer_moves[1] = MaybePos::NONE;

                        return Some((killer_move, KILLER_MOVE_SCORE));
                    }

                    if Self::has_open_four(state) {
                        generate_defend_open_four_moves(state, &mut self.moves);
                        self.stage = MoveStage::DefendFour;
                    } else {
                        generate_neighbors_moves(state, &mut self.moves);
                        self.stage = MoveStage::Neighbor;
                    }
                },
                MoveStage::DefendFour | MoveStage::Neighbor => {
                    return self.moves.consume_best();
                },
                MoveStage::Done => break None
            }
        }
    }

    fn has_open_four(state: &GameState) -> bool {
        let total_fours = match state.board.player_color {
            Color::Black => {
                let mut total_fours = state.board.patterns.counts.global
                    .white.open_fours as u32;

                total_fours -= state.board.patterns.forbidden_field.iter_hot_pos()
                    .map(|pos|
                        state.board.patterns.field.black[pos.idx_usize()].count_open_fours()
                    )
                    .sum::<u32>();

                total_fours
            },
            Color::White => {
                state.board.patterns.counts.global
                    .white.open_fours as u32
            }
        };

        total_fours != 0
    }

}
