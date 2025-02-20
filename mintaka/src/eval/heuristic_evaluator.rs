use crate::eval::evaluator::Evaluator;
use rusty_renju::board::Board;
use rusty_renju::board_iter::BoardIterItem;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::rule::ForbiddenKind;
use rusty_renju::notation::value::Eval;
use rusty_renju::pattern::PatternUnit;

pub struct HeuristicWeights {
    pub three: Eval,
    pub four: Eval,
    pub five: Eval,
    pub double_three_fork: Eval,
    pub three_four_fork: Eval,
    pub double_four_fork: Eval,
    pub forbidden: Eval,
}

pub struct HeuristicEvaluator;

impl Default for HeuristicEvaluator {

    fn default() -> Self {
        HeuristicEvaluator
    }

}

impl HeuristicEvaluator {

    const WEIGHTS: HeuristicWeights = HeuristicWeights {
        three: 5,
        four: 3,
        five: 2,
        double_three_fork: 30,
        three_four_fork: 180,
        double_four_fork: 200,
        forbidden: -15,
    };

    fn eval_pattern<const C: Color>(unit: &PatternUnit) -> Eval {
        if unit.has_five() {
            return Self::WEIGHTS.five;
        }

        if unit.has_any_four() {
            if C == Color::White && unit.has_fours() {
                return Self::WEIGHTS.double_four_fork;
            }

            if unit.has_three() {
                return Self::WEIGHTS.three_four_fork;
            }

            return Self::WEIGHTS.four;
        }

        if unit.has_three() {
            if C == Color::White && unit.has_threes() {
                return Self::WEIGHTS.double_three_fork;
            }

            return Self::WEIGHTS.three;
        }

        0
    }

}

impl Evaluator for HeuristicEvaluator {

    fn static_eval(&self, board: &Board) -> Eval {
        let mut score: Eval = 0;

        for item in board.iter_items() {
            score = score.saturating_add(match item {
                BoardIterItem::Pattern(pattern) => {
                    let eval_black = match pattern.forbidden_kind() {
                        Some(ForbiddenKind::DoubleFour | ForbiddenKind::Overline) =>
                            Self::WEIGHTS.forbidden,
                        None => Self::eval_pattern::<{ Color::Black }>(&pattern.black),
                        _ => 0
                    };

                    let eval_white = Self::eval_pattern::<{ Color::White }>(&pattern.white);

                    match board.player_color {
                        Color::Black => eval_black - eval_white,
                        Color::White => eval_white - eval_black
                    }
                }
                _ => 0
            });
        }

        score
    }

}
