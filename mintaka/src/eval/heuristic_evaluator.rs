use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::value::Score;
use rusty_renju::slice_pattern_count::GlobalPatternCount;

pub struct HeuristicEvaluator;

impl Default for HeuristicEvaluator {

    fn default() -> Self {
        HeuristicEvaluator
    }

}

impl Evaluator for HeuristicEvaluator {

    const POLICY_EVALUATION: bool = false;

    fn eval_value(&self, board: &Board) -> Score {
        let score_black = Self::eval_slice_pattern_counts::<{ Color::Black }>(board);
        let score_white = Self::eval_slice_pattern_counts::<{ Color::White }>(board);

        match board.player_color {
            Color::Black => score_black - score_white,
            Color::White => score_white - score_black,
        }
    }

    fn eval_policy(&self, _board: &Board) -> PolicyDistribution {
        unreachable!()
    }

}

impl HeuristicEvaluator {

    fn eval_slice_pattern_counts<const C: Color>(board: &Board) -> Score {
        let mut counts: GlobalPatternCount = board.patterns.counts.global.get::<C>();

        if C == Color::Black {
            for idx in board.patterns.forbidden_field.iter_hot_idx() {
                let pattern = board.patterns.field.black[idx];

                counts.threes -= pattern.count_closed_fours() as i16;
                counts.closed_fours -= pattern.count_closed_fours() as i16;
                counts.open_fours -= pattern.count_open_threes() as i16;
            }
        }

        counts.closed_fours * HeuristicThreatScores::CLOSED_FOUR
            + counts.threes as Score * HeuristicThreatScores::OPEN_THREE
            + counts.open_fours as Score * HeuristicThreatScores::OPEN_FOUR
            + counts.score
    }

}

struct HeuristicThreatScores; impl HeuristicThreatScores {
    const CLOSED_FOUR: Score = 100;
    const OPEN_THREE: Score = 100;
    const OPEN_FOUR: Score = 1000;
}
