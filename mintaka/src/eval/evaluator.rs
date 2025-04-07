use crate::game_state::GameState;
use rusty_renju::notation::value::Score;

pub trait Evaluator {

    fn static_eval(&self, state: &GameState) -> Score;

}
