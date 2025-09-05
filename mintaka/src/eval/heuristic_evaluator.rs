use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::game_state::GameState;
use crate::movegen::move_scores::MoveScores;
use rusty_renju::const_for;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;
use rusty_renju::pattern::Pattern;

#[derive(Clone)]
pub struct HeuristicEvaluator {
    move_scores: MoveScores
}

impl Evaluator for HeuristicEvaluator {

    type EvaluatorParameter = ();

    fn from_state(state: &GameState) -> Self {
        Self {
            move_scores: (&state.board.hot_field).into()
        }
    }

    fn update(&mut self, _state: &GameState) {}

    fn undo(&mut self, _state: &GameState, _color: Color, _pos: Pos) {}

    fn eval_policy(&self, state: &GameState) -> PolicyDistribution {
        let mut policy = [0; pos::BOARD_SIZE];

        let movegen_field = state.movegen_window.movegen_field & !state.board.hot_field;

        let player_pattern_field = state.board.patterns.field.access(state.board.player_color);
        let player_policy_score_lut = POLICY_SCORE_LUT.access(state.board.player_color);

        let opponent_pattern_field = state.board.patterns.field.access(!state.board.player_color);
        let opponent_policy_score_lut = POLICY_SCORE_LUT.access(!state.board.player_color);

        for idx in movegen_field.iter_hot_idx() {
            let neighbor_score = self.move_scores.scores[idx] as Score;

            let distance_score = {
                let distance = state.history.avg_distance_to_recent_actions(Pos::from_index(idx as u8)) as Score;
                (10 - distance) / 2
            };

            let player_pattern_key = encode_policy_key(player_pattern_field[idx]);
            let opponent_pattern_key = encode_policy_key(opponent_pattern_field[idx]);

            let player_pattern_score = player_policy_score_lut[player_pattern_key] as Score;
            let opponent_pattern_score = opponent_policy_score_lut[opponent_pattern_key] as Score;

            policy[idx] = neighbor_score + distance_score + player_pattern_score + opponent_pattern_score / 2;
        }

        policy
    }

    fn eval_value(&self, state: &GameState) -> Score {
        let mut acc_black = 0;

        let eval_field = state.movegen_window.movegen_field & !state.board.hot_field;

        for idx in eval_field.iter_hot_idx() {
            let black_pattern_key = encode_value_key(state.board.patterns.field.black[idx]);
            let white_pattern_key = encode_value_key(state.board.patterns.field.white[idx]);

            acc_black += VALUE_SCORE_LUT.black[black_pattern_key] as Score;
            acc_black -= VALUE_SCORE_LUT.white[white_pattern_key] as Score;
        }

        match state.board.player_color {
            Color::Black => acc_black,
            Color::White => -acc_black
        }
    }

}

const VALUE_SCORE_LUT_SIZE: usize = (0b1 << 8) + 1;
const VALUE_SCORE_LUT_OVERLINE_MASK: u32 = !(u32::MAX << 8);

// (open_fours(1), fours(2), open_threes(2), potential(3)
// overline override for full-bits
// total 8 bits
fn encode_value_key(pattern: Pattern) -> usize {
    let mut pattern_key = 0;

    pattern_key |= pattern.has_open_four() as u32;
    pattern_key |= (pattern.count_total_fours() & 0b11) << 1;
    pattern_key |= (pattern.count_open_threes() & 0b11) << 3;
    pattern_key |= (pattern.count_potentials() & 0b111) << 5;
    pattern_key |= (pattern.has_overline() as u32) * VALUE_SCORE_LUT_OVERLINE_MASK;

    pattern_key as usize
}

type ValueScoreLut = [i16; VALUE_SCORE_LUT_SIZE];

const VALUE_SCORE_LUT: ColorContainer<ValueScoreLut> = build_value_score_lut();

const fn build_value_score_lut() -> ColorContainer<ValueScoreLut> {
    let mut acc = ColorContainer::new(
        [0; VALUE_SCORE_LUT_SIZE],
        [0; VALUE_SCORE_LUT_SIZE]
    );

    const fn flash_score_variants(color: Color, lut: &mut ValueScoreLut) {
        const_for!(pattern_key in 0, VALUE_SCORE_LUT_SIZE; {
            let has_open_four = (pattern_key & 0b1) == 0b1;
            let fours = (pattern_key >> 1) & 0b11;
            let closed_fours = fours.saturating_sub(has_open_four as usize);
            let open_threes = (pattern_key >> 3) & 0b11;
            let mut potentials = (pattern_key >> 5) & 0b111;

            if potentials > 4 {
                potentials = 4;
            }

            let mut acc = 0;

            match color {
                Color::Black => {
                    if pattern_key == VALUE_SCORE_LUT_OVERLINE_MASK as usize {
                        acc = HeuristicValueScores::OVERLINE_FORBID;
                    } else {
                        if fours > 1 {
                            acc = HeuristicValueScores::DOUBLE_FOUR_FORBID;
                        } else if open_threes > 1 {
                            acc = HeuristicValueScores::DOUBLE_THREE_FORBID;
                        } else if has_open_four {
                            acc = HeuristicValueScores::OPEN_FOUR;
                        } else if fours == 1 && open_threes == 1 {
                            acc = HeuristicValueScores::THREE_FOUR_FORK;
                        } else if open_threes == 1 {
                            acc = HeuristicValueScores::OPEN_THREE;
                        } else if closed_fours == 1 {
                            acc = HeuristicValueScores::CLOSED_FOUR;
                        }

                        let potential_score = HeuristicValueScores::POTENTIAL[potentials];

                        if acc > 0 {
                            acc += potential_score * 2;
                        } else {
                            acc += potential_score;
                        }
                    }
                },
                Color::White => {
                    if has_open_four {
                        acc = HeuristicValueScores::OPEN_FOUR;
                    } else if fours > 1 {
                        acc = HeuristicValueScores::DOUBLE_FOUR_FORK;
                    } else if fours == 1 && open_threes > 0 {
                        acc = HeuristicValueScores::THREE_FOUR_FORK;
                    } else if open_threes > 1 {
                        acc = HeuristicValueScores::DOUBLE_THREE_FORK
                    } else if open_threes == 1 {
                        acc = HeuristicValueScores::OPEN_THREE;
                    } else if fours == 1 {
                        acc = HeuristicValueScores::CLOSED_FOUR;
                    }

                    let potential_score = HeuristicValueScores::POTENTIAL[potentials];

                    if acc > 0 {
                        acc += potential_score * 2;
                    } else {
                        acc += potential_score;
                    }
                }
            }

            lut[pattern_key] = acc;
        });
    }

    flash_score_variants(Color::Black, &mut acc.black);
    flash_score_variants(Color::White, &mut acc.white);

    acc
}

struct HeuristicValueScores; impl HeuristicValueScores {
    const POTENTIAL: [i16; 5]       = [0, 1, 8, 16, 40];

    const OPEN_THREE: i16           = 22;
    const CLOSED_FOUR: i16          = 8;
    const OPEN_FOUR: i16            = 2000;

    const THREE_FOUR_FORK: i16      = 1800;
    const DOUBLE_THREE_FORK: i16    = 400;
    const DOUBLE_FOUR_FORK: i16     = 2000;

    const OVERLINE_FORBID: i16      = -100;
    const DOUBLE_FOUR_FORBID: i16   = -50;
    const DOUBLE_THREE_FORBID: i16  = -40;
}

const POLICY_SCORE_LUT_SIZE: usize = (0b1 << 8) + 1;
const POLICY_SCORE_LUT_OVERLINE_MASK: u32 = !(u32::MAX << 8);

// (open_fours(1), fours(2), open_threes(2), potential(3)
// overline override for full-bits
// total 8 bits
fn encode_policy_key(pattern: Pattern) -> usize {
    let mut pattern_key = 0;

    pattern_key |= pattern.has_open_four() as u32;
    pattern_key |= (pattern.count_total_fours() & 0b11) << 1;
    pattern_key |= (pattern.count_open_threes() & 0b11) << 3;
    pattern_key |= (pattern.count_potentials() & 0b111) << 5;
    pattern_key |= (pattern.has_overline() as u32) * POLICY_SCORE_LUT_OVERLINE_MASK;

    pattern_key as usize
}

type PolicyScoreLut = [i8; POLICY_SCORE_LUT_SIZE];

const POLICY_SCORE_LUT: ColorContainer<PolicyScoreLut> = build_pattern_score_lut();

const fn build_pattern_score_lut() -> ColorContainer<PolicyScoreLut> {
    let mut acc = ColorContainer::new(
        [0; POLICY_SCORE_LUT_SIZE],
        [0; POLICY_SCORE_LUT_SIZE]
    );

    const fn flash_score_variants(color: Color, lut: &mut PolicyScoreLut) {
        const_for!(pattern_key in 0, POLICY_SCORE_LUT_SIZE; {
            let has_open_four = (pattern_key & 0b1) == 0b1;
            let fours = (pattern_key >> 1) & 0b11;
            let closed_fours = fours.saturating_sub(has_open_four as usize);
            let open_threes = (pattern_key >> 3) & 0b11;
            let mut potentials = (pattern_key >> 5) & 0b111;

            if potentials > 4 {
                potentials = 4;
            }

            lut[pattern_key] = match color {
                Color::Black => {
                    if pattern_key == POLICY_SCORE_LUT_OVERLINE_MASK as usize {
                        HeuristicPolicyScores::OVERLINE_FORBID
                    } else {
                        let acc = if fours > 1 {
                            HeuristicPolicyScores::DOUBLE_FOUR_FORBID
                        } else if open_threes > 1 {
                            HeuristicPolicyScores::DOUBLE_THREE_FORBID
                        } else if has_open_four {
                            HeuristicPolicyScores::OPEN_FOUR
                        } else if closed_fours == 1 && open_threes == 1 {
                            HeuristicPolicyScores::THREE_FOUR_FORK
                        } else if open_threes == 1 {
                            HeuristicPolicyScores::OPEN_THREE
                        } else if closed_fours == 1 {
                            HeuristicPolicyScores::CLOSED_FOUR
                        } else {
                            0
                        };

                        if acc > 0 && acc < i8::MAX {
                            acc.saturating_add(HeuristicPolicyScores::POTENTIAL[potentials] * 2)
                        } else {
                            acc
                        }
                    }
                },
                Color::White => {
                    let acc = if has_open_four {
                        HeuristicPolicyScores::OPEN_FOUR
                    } else if fours > 1 {
                        HeuristicPolicyScores::DOUBLE_FOUR_FORK
                    } else if fours > 0 && open_threes > 0 {
                        HeuristicPolicyScores::THREE_FOUR_FORK
                    } else if open_threes > 1 {
                        HeuristicPolicyScores::DOUBLE_THREE_FORK
                    } else if open_threes == 1 {
                        HeuristicPolicyScores::OPEN_THREE
                    } else if closed_fours == 1 {
                        HeuristicPolicyScores::CLOSED_FOUR
                    } else {
                        0
                    };

                    if acc > 0 && acc < i8::MAX {
                        acc.saturating_add(HeuristicPolicyScores::POTENTIAL[potentials] * 2)
                    } else {
                        acc
                    }
                }
            }
        });
    }

    flash_score_variants(Color::Black, &mut acc.black);
    flash_score_variants(Color::White, &mut acc.white);

    acc
}

struct HeuristicPolicyScores; impl HeuristicPolicyScores {
    const POTENTIAL: [i8; 5] = [0, 1, 8, 25, 50];

    const OPEN_THREE: i8 = 20;
    const CLOSED_FOUR: i8 = 10;
    const OPEN_FOUR: i8 = i8::MAX;

    const DOUBLE_THREE_FORK: i8 = 70;
    const THREE_FOUR_FORK: i8 = i8::MAX;
    const DOUBLE_FOUR_FORK: i8 = i8::MAX;

    const DOUBLE_THREE_FORBID: i8 = i8::MIN;
    const DOUBLE_FOUR_FORBID: i8 = i8::MIN;
    const OVERLINE_FORBID: i8 = i8::MIN;
}
