use crate::eval::evaluator::Evaluator;
use rusty_renju::board::Board;
use rusty_renju::board_iter::BoardIterItem;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::node::Score;

pub struct HeuristicEvaluator;

impl Default for HeuristicEvaluator {

    fn default() -> Self {
        HeuristicEvaluator
    }

}

impl Evaluator for HeuristicEvaluator {

    fn eval(&self, board: &Board) -> Score {
        let mut score: Score = 0;

        for item in board.iter_items() {
            score += match item {
                BoardIterItem::Stone(stone) => {
                    0
                }
                BoardIterItem::Pattern(pattern) => {
                    let (player_pattern, opponent_pattern) = match board.player_color {
                        Color::Black => (pattern.black_unit, pattern.white_unit),
                        Color::White => (pattern.white_unit, pattern.black_unit)
                    };

                    0
                }
            }

        }

        score
    }

}
