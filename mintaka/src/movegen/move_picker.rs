use crate::eval::scores;
use crate::game_state::GameState;
use crate::movegen::move_generator::{generate_defend_three_moves, generate_neighbors_moves, is_open_four_available};
use crate::movegen::move_list::MoveList;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::Score;

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
    killer_move: MaybePos,
}

impl MovePicker {

    pub fn new(
        tt_move: MaybePos,
        killer_move: MaybePos,
    ) -> Self {
        Self {
            stage: MoveStage::TT,
            moves: MoveList::default(),
            tt_move,
            killer_move,
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
                    return Some((self.tt_move.unwrap(), scores::TT_MOVE));
                }

                self.next(state)
            },
            MoveStage::Killer => {
                if is_open_four_available(&state.board) {
                    generate_defend_three_moves(state, &mut self.moves);
                    self.stage = MoveStage::DefendFour;
                } else {
                    generate_neighbors_moves(state, &mut self.moves);
                    self.stage = MoveStage::Neighbor;
                }

                if self.killer_move.is_some() {
                    return Some((self.killer_move.unwrap(), 0));
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
