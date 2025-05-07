use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::game_state::GameState;
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

    fn eval_value(&self, state: &GameState) -> Score {
        let score_black = Self::eval_slice_pattern_counts::<{ Color::Black }>(state);
        let score_white = Self::eval_slice_pattern_counts::<{ Color::White }>(state);

        match state.board.player_color {
            Color::Black => score_black - score_white,
            Color::White => score_white - score_black,
        }
    }

    fn eval_policy(&self, _state: &GameState) -> PolicyDistribution {
        unreachable!()
    }

}

impl HeuristicEvaluator {

    fn eval_slice_pattern_counts<const C: Color>(state: &GameState) -> Score {
        let mut counts: GlobalPatternCount = state.board.patterns.counts.global.get::<C>();

        if C == Color::Black {
            for idx in state.board.patterns.forbidden_field.iter_hot_idx() {
                let pattern = state.board.patterns.field.black[idx];

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
