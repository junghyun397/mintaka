use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use rusty_renju::board_iter::BoardIterItem;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::rule::ForbiddenKind;
use rusty_renju::notation::value::Score;
use rusty_renju::pattern::{Pattern, PatternCount};

struct HeuristicWeights {
    pub three: isize,
    pub four: isize,
    pub open_four: isize,
    pub five: isize,
    pub double_three_fork: isize,
    pub three_four_fork: isize,
    pub double_four_fork: isize,
    pub soft_forbidden: isize,
    pub hard_forbidden: isize,
}

enum PatternAssign {
    Three = 0,
    Four = 1,
    OpenFour = 2,
    Five = 3,
    DoubleThreeFork = 4,
    ThreeFourFork = 5,
    DoubleFourFork = 6,
    SoftForbidden = 7,
    HardForbidden = 8,
}

type PatternAcc = [isize; 9];

pub struct HeuristicEvaluator;

impl Default for HeuristicEvaluator {

    fn default() -> Self {
        HeuristicEvaluator
    }

}

impl HeuristicEvaluator {

    const WEIGHTS: HeuristicWeights = HeuristicWeights {
        three: 5,
        four: 2,
        open_four: 10,
        five: 8,
        double_three_fork: 30,
        three_four_fork: 180,
        double_four_fork: 200,
        soft_forbidden: 1,
        hard_forbidden: -15,
    };

    fn score_acc(acc: &PatternAcc) -> isize {
        acc[PatternAssign::Three as usize] * Self::WEIGHTS.three
            + acc[PatternAssign::Four as usize] * Self::WEIGHTS.four
            + acc[PatternAssign::OpenFour as usize] * Self::WEIGHTS.open_four
            + acc[PatternAssign::Five as usize] * Self::WEIGHTS.five
            + acc[PatternAssign::DoubleThreeFork as usize] * Self::WEIGHTS.double_three_fork
            + acc[PatternAssign::ThreeFourFork as usize] * Self::WEIGHTS.three_four_fork
            + acc[PatternAssign::DoubleFourFork as usize] * Self::WEIGHTS.double_four_fork
            + acc[PatternAssign::SoftForbidden as usize] * Self::WEIGHTS.soft_forbidden
            + acc[PatternAssign::HardForbidden as usize] * Self::WEIGHTS.hard_forbidden
    }

    fn eval_pattern<const C: Color>(unit: &Pattern, acc: &mut PatternAcc) {
        if unit.has_five() {
            acc[PatternAssign::Five as usize] += 1;
        }

        match unit.count_fours() {
            PatternCount::Single => {
                if unit.has_open_four() {
                    acc[PatternAssign::OpenFour as usize] += 1;
                } else if unit.has_three() {
                    acc[PatternAssign::ThreeFourFork as usize] += 1;
                } else {
                    acc[PatternAssign::Four as usize] += 1;
                }

                return;
            }
            PatternCount::Multiple => {
                if C == Color::White {
                    acc[PatternAssign::DoubleFourFork as usize] += 1;
                    return;
                }
            },
            _ => {}
        }

        match C {
            Color::Black => {
                if unit.has_three() {
                    acc[PatternAssign::Three as usize] += 1;
                }
            },
            Color::White => match unit.count_threes() {
                PatternCount::Single => {
                    acc[PatternAssign::Three as usize] += 1;
                },
                PatternCount::Multiple => {
                    acc[PatternAssign::DoubleThreeFork as usize] += 1;
                },
                _ => {}
            }
        }
    }

}

impl Evaluator for HeuristicEvaluator {

    fn static_eval(&self, state: &GameState) -> Score {
        let mut acc_black = [0; 9];
        let mut acc_white = [0; 9];

        for item in state.board.iter_items() {
            if let BoardIterItem::Pattern(pattern) = item {
                match pattern.black.forbidden_kind() {
                    Some(ForbiddenKind::DoubleThree) => {
                        acc_black[PatternAssign::SoftForbidden as usize] += 1;
                    },
                    Some(ForbiddenKind::DoubleFour | ForbiddenKind::Overline) => {
                        acc_black[PatternAssign::HardForbidden as usize] += 1;
                    },
                    None => {
                        Self::eval_pattern::<{ Color::Black }>(&pattern.black, &mut acc_black);
                    }
                };

                Self::eval_pattern::<{ Color::White }>(&pattern.white, &mut acc_white);
            }
        }

        let black_win = acc_black[PatternAssign::Five as usize] > 1;
        let white_win = acc_white[PatternAssign::Five as usize] > 1;

        let score = match state.board.player_color {
            Color::Black => {
                if black_win {
                    isize::MAX
                } else if white_win {
                    isize::MIN
                } else {
                    Self::score_acc(&acc_black) - Self::score_acc(&acc_white)
                }
            },
            Color::White => {
                if white_win {
                    isize::MAX
                } else if black_win {
                    isize::MIN
                } else {
                    Self::score_acc(&acc_white) - Self::score_acc(&acc_black)
                }
            }
        };

        score.clamp(-10000, 10000) as Score
    }

}
