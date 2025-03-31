use crate::game_state::GameState;
use crate::movegen::move_generator::{generate_defend_three_moves, generate_neighbors_moves, is_open_four_available};
use crate::movegen::move_list::MoveList;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::notation::pos::Pos;

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
    pub stage: MoveStage,
    moves: MoveList,
    index: usize,
    tt_move: Option<Pos>,
    killer_move: Option<Pos>,
}

impl MovePicker {

    pub fn new(
        td: &ThreadData<impl ThreadType>,
        tt_move: Option<Pos>,
        killer_move: Option<Pos>,
    ) -> Self {
        Self {
            stage: MoveStage::TT,
            moves: MoveList::default(),
            index: 0,
            tt_move,
            killer_move,
        }
    }

    pub fn next(
        &mut self,
        td: &ThreadData<impl ThreadType>,
        state: &GameState,
    ) -> Option<(Pos, i32)> {
        if self.stage == MoveStage::TT {
            self.stage = MoveStage::Killer;

            if let Some(tt_move) = self.tt_move {
                return Some((tt_move, TT_MOVE_SCORE));
            }
        }

        if self.stage == MoveStage::Killer {
            self.stage = if is_open_four_available(&state.board) {
                generate_defend_three_moves(&state.board, &mut self.moves);

                MoveStage::DefendFour
            } else {
                generate_neighbors_moves(&state.board, &state.movegen_window, &mut self.moves);

                MoveStage::Neighbor
            };

            if let Some(killer_move) = self.killer_move {
                return Some((killer_move, 0));
            }
        }

        if self.stage == MoveStage::DefendFour || self.stage == MoveStage::Neighbor {
            let next_move = self.pick_next();

            if next_move.is_none() {
                self.stage = MoveStage::Done;
            }

            return next_move;
        }

        None
    }

    fn pick_next(&mut self) -> Option<(Pos, i32)> {
        if self.index >= self.moves.len() {
            return None;
        }

        let mut best_idx = 0;
        let mut best_score = i32::MIN;

        for (idx, &(pos, score)) in self.moves.iter() {
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        Some(self.moves.consume(best_idx))
    }

}
