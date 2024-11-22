use crate::eval::evaluator::Evaluator;
use crate::memo::tt_entry::Score;
use mintaka::board::Board;
use mintaka::board_iter::BoardIterItem;
use mintaka::notation::color::Color;

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
                BoardIterItem::Stone(stine) => {
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
