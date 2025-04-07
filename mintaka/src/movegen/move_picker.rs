use crate::eval::scores;
use crate::game_state::GameState;
use crate::movegen::move_generator::{generate_defend_three_moves, generate_neighbors_moves, is_open_four_available};
use crate::movegen::move_list::MoveList;
use crate::search_frame::KILLER_MOVE_SLOTS;
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
        match self.stage {
            MoveStage::TT => {
                self.stage = MoveStage::Killer;

                if self.tt_move.is_some() {
                    return Some((self.tt_move.unwrap(), scores::TT_MOVE));
                }

                self.next(state)
            },
            MoveStage::Killer => {
                for killer_move in self.killer_moves.iter_mut()
                    .filter(|action| action.is_some())
                {
                        let pos = killer_move.unwrap();
                        *killer_move = MaybePos::NONE;

                        return Some((pos, scores::KILLER_MOVE));
                }

                if is_open_four_available(&state.board) {
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
