use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::game_state::GameState;
use crate::movegen::move_scores::MoveScores;
use rusty_renju::const_for;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;
use rusty_renju::pattern::Pattern;
use rusty_renju::slice_pattern_count::GlobalPatternCount;

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

        let pattern_field = state.board.patterns.field.access(state.board.player_color);
        let policy_score_lut = POLICY_SCORE_LUT.access(state.board.player_color);

        for idx in movegen_field.iter_hot_idx() {
            let neighbor_score = self.move_scores.scores[idx] as Score;

            let distance_score = {
                let distance = state.history.avg_distance_to_recent_actions(Pos::from_index(idx as u8)) as Score;
                (10 - distance) / 2
            };

            let pattern_score = probe_score_table_lut(policy_score_lut, pattern_field[idx]) as Score;

            policy[idx] = neighbor_score + distance_score + pattern_score;
        }

        policy
    }

    fn eval_value(&self, state: &GameState) -> Score {
        let eval_black = Self::eval_slice_pattern_counts::<{ Color::Black }>(state);
        let eval_white = Self::eval_slice_pattern_counts::<{ Color::White }>(state);

        match state.board.player_color {
            Color::Black => eval_black - eval_white,
            Color::White => eval_white - eval_black,
        }
    }

}

struct HeuristicThreatScores; impl HeuristicThreatScores {
    const CLOSED_FOUR: Score = 100;
    const OPEN_THREE: Score = 100;
    const OPEN_FOUR: Score = 1000;
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

        let mut score = 0;

        if counts.open_fours > 1 { // multiple open-fours
            score += 10000;
        }

        score += counts.closed_fours as Score * HeuristicThreatScores::CLOSED_FOUR;
        score += counts.threes as Score * HeuristicThreatScores::OPEN_THREE;
        score += counts.open_fours as Score * HeuristicThreatScores::OPEN_FOUR;
        score += counts.score as Score;

        score
    }

}

fn probe_score_table_lut(lut: &PolicyScoreLut, pattern: Pattern) -> i8 {
    let mut pattern_key = pattern.count_closed_fours() & 0b11;

    pattern_key |= (pattern.count_open_fours() & 0b11) << 2;
    pattern_key |= (pattern.count_open_threes() & 0b11) << 4;
    pattern_key |= (pattern.has_close_three() as u32) << 6;
    pattern_key |= (pattern.has_overline() as u32) * 127;

    lut[pattern_key as usize]
}

struct HeuristicPolicyScores; impl HeuristicPolicyScores {
    const OPEN_THREE: i8 = 15;
    const CLOSE_THREE: i8 = 0;
    const CLOSED_FOUR: i8 = 10;
    const OPEN_FOUR: i8 = i8::MAX;
    const DOUBLE_THREE_FORK: i8 = 30;
    const THREE_FOUR_FORK: i8 = i8::MAX;
    const DOUBLE_FOUR_FORK: i8 = i8::MAX;
    const DOUBLE_THREE_FORBID: i8 = 4;
    const DOUBLE_FOUR_FORBID: i8 = -3;
    const OVERLINE_FORBID: i8 = -30;
}

type PolicyScoreLut = [i8; 0b1 << 7];
const POLICY_SCORE_LUT: ColorContainer<PolicyScoreLut> = build_pattern_score_lut();

const fn build_pattern_score_lut() -> ColorContainer<PolicyScoreLut> {
    let mut acc = ColorContainer::new(
        [0; 0b1 << 7],
        [0; 0b1 << 7]
    );

    const fn flash_score_variants(
        color: Color,
        lut: &mut PolicyScoreLut,
    ) {
        const_for!(pattern_key in 0, 0b1 << 7; {
            let closed_fours = pattern_key & 0b11;
            let open_fours = (pattern_key & 0b1100) >> 2;
            let open_threes = (pattern_key & 0b110000) >> 4;
            let close_threes = (pattern_key & 0b1000000) >> 6;

            lut[pattern_key] = match color {
                Color::Black => {
                    if pattern_key == 127 {
                        HeuristicPolicyScores::OVERLINE_FORBID
                    } else if closed_fours + open_fours > 1 {
                        HeuristicPolicyScores::DOUBLE_FOUR_FORBID
                    } else if open_threes > 1 {
                        HeuristicPolicyScores::DOUBLE_THREE_FORBID
                    } else if open_fours == 1 {
                        HeuristicPolicyScores::OPEN_FOUR
                    } else if closed_fours == 1 && open_threes == 1 {
                        HeuristicPolicyScores::THREE_FOUR_FORK
                    } else if open_threes == 1 {
                        HeuristicPolicyScores::OPEN_THREE
                    } else if close_threes != 0 {
                        HeuristicPolicyScores::CLOSE_THREE
                    } else {
                        0
                    }
                },
                Color::White => {
                    if open_fours > 0 {
                        HeuristicPolicyScores::OPEN_FOUR
                    } else if closed_fours > 1 {
                        HeuristicPolicyScores::DOUBLE_FOUR_FORK
                    } else if closed_fours > 0 && open_threes > 0 {
                        HeuristicPolicyScores::THREE_FOUR_FORK
                    } else if open_threes > 1 {
                        HeuristicPolicyScores::DOUBLE_THREE_FORK
                    } else if open_threes == 1 {
                        HeuristicPolicyScores::OPEN_THREE
                    } else if closed_fours == 1 {
                        HeuristicPolicyScores::CLOSED_FOUR
                    } else if close_threes != 0 {
                        HeuristicPolicyScores::CLOSE_THREE
                    } else {
                        0
                    }
                }
            }
        });
    }

    flash_score_variants(Color::Black, &mut acc.black);
    flash_score_variants(Color::White, &mut acc.white);

    acc
}
