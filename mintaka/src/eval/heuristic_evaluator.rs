use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::game_state::GameState;
use crate::movegen::move_scores::MoveScores;
use rusty_renju::board::Board;
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

    fn from_state(state: &GameState) -> Self {
        Self {
            move_scores: (&state.board.hot_field).into()
        }
    }

    fn update(&mut self, state: &GameState) {
    }

    fn undo(&mut self, state: &GameState, color: Color, pos: Pos) {
    }

    fn eval_policy(&self, state: &GameState) -> PolicyDistribution {
        let mut policy = [0; pos::BOARD_SIZE];

        for (idx, &score) in self.move_scores.scores.iter().enumerate().take(pos::BOARD_SIZE) {
            let distance = state.history.avg_distance_to_recent_actions(Pos::from_index(idx as u8)) as Score;

            policy[idx] = (16 - distance) + score as Score;
        }

        policy
    }

    fn eval_value(&self, state: &GameState) -> Score {
        let black_score = Self::eval_slice_pattern_counts::<{ Color::Black }>(&state.board);
        let white_score = Self::eval_slice_pattern_counts::<{ Color::White }>(&state.board);

        match state.board.player_color {
            Color::Black => black_score - white_score,
            Color::White => white_score - black_score,
        }
    }

}

struct HeuristicThreatScores; impl HeuristicThreatScores {
    const CLOSED_FOUR: Score = 100;
    const OPEN_THREE: Score = 100;
    const OPEN_FOUR: Score = 1000;
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

        counts.closed_fours as Score * HeuristicThreatScores::CLOSED_FOUR
            + counts.threes as Score * HeuristicThreatScores::OPEN_THREE
            + counts.open_fours as Score * HeuristicThreatScores::OPEN_FOUR
            + counts.score as Score
    }

    fn eval_pattern(state: &GameState) -> Score {
        let mut black_score = 0;
        let mut white_score = 0;

        for idx in state.movegen_window.movegen_field.iter_hot_idx() {
            black_score +=
                probe_score_table_lut::<{ Color::Black }>(state.board.patterns.field.get::<{ Color::Black }>()[idx]) as Score;
            white_score -=
                probe_score_table_lut::<{ Color::White }>(state.board.patterns.field.get::<{ Color::White }>()[idx]) as Score;
        }

        match state.board.player_color {
            Color::Black => black_score - white_score,
            Color::White => white_score - black_score,
        }
    }

    fn eval_slice(state: &GameState) -> Score {
        let mut player_score = 0;

        0
    }

}

fn probe_score_table_lut<const C: Color>(pattern: Pattern) -> i8 {
    let mut pattern_key = pattern.count_closed_fours() & 0b11;

    pattern_key |= (pattern.count_open_fours() & 0b11) << 2;
    pattern_key |= (pattern.count_open_threes() & 0b11) << 4;
    pattern_key |= (pattern.has_close_three() as u32) << 6;
    pattern_key |= (pattern.has_overline() as u32) * 127;

    PATTERN_SCORE_LUT.get_ref::<C>()[pattern_key as usize]
}

struct HeuristicPositionScores; impl HeuristicPositionScores {
    const OPEN_THREE: i8 = 5;
    const CLOSE_THREE: i8 = 0;
    const CLOSED_FOUR: i8 = 2;
    const OPEN_FOUR: i8 = i8::MAX;
    const DOUBLE_THREE_FORK: i8 = 30;
    const THREE_FOUR_FORK: i8 = i8::MAX;
    const DOUBLE_FOUR_FORK: i8 = i8::MAX;
    const DOUBLE_THREE_FORBID: i8 = 1;
    const DOUBLE_FOUR_FORBID: i8 = -3;
    const OVERLINE_FORBID: i8 = -3;
}

const PATTERN_SCORE_LUT: ColorContainer<[i8; 0b1 << 7]> = build_pattern_score_lut();

const fn build_pattern_score_lut() -> ColorContainer<[i8; 128]> {
    let mut acc = ColorContainer::new(
        [0; 0b1 << 7],
        [0; 0b1 << 7]
    );

    const fn flash_score_variants(
        color: Color,
        lut: &mut [i8; 0b1 << 7],
    ) {
        const_for!(pattern_key in 0, 0b1 << 7; {
            let closed_fours = pattern_key & 0b11;
            let open_fours = (pattern_key & 0b1100) >> 2;
            let open_threes = (pattern_key & 0b110000) >> 4;
            let close_threes = (pattern_key & 0b1000000) >> 6;

            lut[pattern_key] = match color {
                Color::Black => {
                    if pattern_key == 127 {
                        HeuristicPositionScores::OVERLINE_FORBID
                    } else if closed_fours + open_fours > 1 {
                        HeuristicPositionScores::DOUBLE_FOUR_FORBID
                    } else if open_threes > 1 {
                        HeuristicPositionScores::DOUBLE_THREE_FORBID
                    } else if open_fours == 1 {
                        HeuristicPositionScores::OPEN_FOUR
                    } else if closed_fours == 1 && open_threes == 1 {
                        HeuristicPositionScores::THREE_FOUR_FORK
                    } else if open_threes == 1 {
                        HeuristicPositionScores::OPEN_THREE
                    } else if close_threes != 0 {
                        HeuristicPositionScores::CLOSE_THREE
                    } else {
                        0
                    }
                },
                Color::White => {
                    if open_fours > 0 {
                        HeuristicPositionScores::OPEN_FOUR
                    } else if closed_fours > 1 {
                        HeuristicPositionScores::DOUBLE_FOUR_FORK
                    } else if closed_fours > 0 && open_threes > 0 {
                        HeuristicPositionScores::THREE_FOUR_FORK
                    } else if open_threes > 1 {
                        HeuristicPositionScores::DOUBLE_THREE_FORK
                    } else if open_threes == 1 {
                        HeuristicPositionScores::OPEN_THREE
                    } else if closed_fours == 1 {
                        HeuristicPositionScores::CLOSED_FOUR
                    } else if close_threes != 0 {
                        HeuristicPositionScores::CLOSE_THREE
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
