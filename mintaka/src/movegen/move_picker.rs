use crate::game_state::GameState;
use crate::movegen::move_generator::{generate_defend_three_moves, generate_neighbors_moves, is_open_four_available};
use crate::movegen::move_list::MoveList;
use rusty_renju::notation::pos::{MaybePos, Pos};

const TT_MOVE_SCORE: i32 = i32::MAX - 5000;

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
    ) -> Option<(Pos, i32)> {
        match self.stage {
            MoveStage::TT => {
                self.stage = MoveStage::Killer;

                if self.tt_move.is_some() {
                    return Some((self.tt_move.unwrap(), TT_MOVE_SCORE));
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
                let next_move = self.pick_next();

                if next_move.is_none() {
                    self.stage = MoveStage::Done;
                }

                next_move
            },
            MoveStage::Done => None
        }
    }

    fn pick_next(&mut self) -> Option<(Pos, i32)> {
        if self.moves.is_empty() {
            return None;
        }

        let mut best_idx = 0;
        let mut best_score = i32::MIN;

        for (idx, &(pos, score)) in self.moves.iter().enumerate() {
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        Some(self.moves.consume(best_idx))
    }

}
