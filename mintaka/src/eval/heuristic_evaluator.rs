use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::state::GameState;
use crate::movegen::move_scores::MoveScores;
use rusty_renju::bitfield::Bitfield;
use rusty_renju::board::Board;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::score::{Score, Scores};
use rusty_renju::pattern::Pattern;
use rusty_renju::{const_for, pattern};

#[derive(Clone)]
pub struct HeuristicEvaluator {
    move_scores: MoveScores,
    eval_history: [(HashKey, Score); pos::BOARD_SIZE]
}

impl Evaluator for HeuristicEvaluator {

    type EvaluatorParameter = ();

    fn from_state(state: &GameState) -> Self {
        Self {
            move_scores: (&state.board.hot_field).into(),
            eval_history: [(HashKey::INVALID, Score::DRAW); pos::BOARD_SIZE]
        }
    }

    fn update(&mut self, _state: &GameState) {}

    fn undo(&mut self, _state: &GameState, _color: Color, _pos: Pos) {}

    fn eval_policy(&mut self, state: &GameState) -> PolicyDistribution {
        let mut policy = [0; pattern::PATTERN_SIZE];

        let movegen_field = state.movegen_window.movegen_field & state.board.legal_field();

        let player_pattern_field = &state.board.patterns.field[state.board.player_color];
        let player_policy_score_lut = &POLICY_SCORE_LUT[state.board.player_color];

        let opponent_pattern_field = &state.board.patterns.field[!state.board.player_color];
        let opponent_policy_score_lut = &POLICY_SCORE_LUT[!state.board.player_color];

        for idx in movegen_field.iter_hot_idx() {
            let neighbor_score = self.move_scores.scores[idx] as i16;

            let distance_score = {
                let distance = state.history.avg_distance_to_recent_actions(Pos::from_index(idx as u8)) as i16;
                (10 - distance) / 2
            };

            let player_pattern_key = encode_policy_key(player_pattern_field[idx]);
            let opponent_pattern_key = encode_policy_key(opponent_pattern_field[idx]);

            let player_pattern_score = player_policy_score_lut[player_pattern_key] as i16;
            let opponent_pattern_score = opponent_policy_score_lut[opponent_pattern_key] as i16;

            policy[idx] = neighbor_score + distance_score + player_pattern_score + opponent_pattern_score / 2;
        }

        policy
    }

    fn eval_value(&mut self, state: &GameState) -> Score {
        if state.is_empty() {
            return 0;
        }

        let parent_score =
            if state.len() > 1 {
                if let Some(&(hash_key, score)) = self.eval_history.get(state.len() - 2)
                    && hash_key == state.board.hash_key.set(!state.board.player_color, state.history.recent_action().unwrap())
                {
                    -score
                } else {
                    let parent_board = state.board.unset(state.history.recent_action().unwrap());

                    let parent_score = self.eval_board_value(&parent_board, !parent_board.hot_field);

                    self.eval_history[state.len() - 2] = (parent_board.hash_key, parent_score);

                    -parent_score
                }
            } else {
                0
            };

        let movegen_field = state.movegen_window.movegen_field & !state.board.hot_field;

        let current_score = self.eval_board_value(&state.board, movegen_field);

        self.eval_history[state.len() - 1] = (state.board.hash_key, current_score);

        (current_score + parent_score) / 2
    }

}

impl HeuristicEvaluator {

    fn eval_board_value(&self, board: &Board, eval_window: Bitfield) -> Score {
        let mut value_black = 0;
        let mut tactical_black = 0;
        let mut tactical_white = 0;

        for idx in eval_window.iter_hot_idx() {
            let black_pattern = board.patterns.field[Color::Black][idx];
            let white_pattern = board.patterns.field[Color::White][idx];

            let black_pattern_key = encode_value_key(black_pattern);
            let white_pattern_key = encode_value_key(white_pattern);

            let (value_local_black, tactical_local_black) = VALUE_SCORE_LUT[Color::Black][black_pattern_key];
            let (value_local_white, tactical_local_white) = VALUE_SCORE_LUT[Color::White][white_pattern_key];

            value_black += value_local_black as Score;
            value_black -= value_local_white as Score;

            tactical_black += tactical_local_black;
            tactical_white += tactical_local_white;
        }

        let (score, tactical_points, opponent_tactical_points) = match board.player_color {
            Color::Black => (value_black, tactical_black, tactical_white),
            Color::White => (-value_black, tactical_white, tactical_black)
        };

        let tactical_black = Self::eval_tactical_value(board, tactical_points, opponent_tactical_points);

        score + tactical_black
    }

    fn eval_tactical_value(board: &Board, mut tactical_points: u16, mut opponent_tactical_points: u16) -> Score {
        tactical_points += board.patterns.counts.slice[board.player_color].total_open_four_structs_unchecked() as u16;
        opponent_tactical_points += board.patterns.counts.slice[!board.player_color].total_open_four_structs_unchecked() as u16;

        if tactical_points > 0 {
            return 10000;
        } else if opponent_tactical_points > 1 {
            return -10000;
        }

        0
    }

}

const VALUE_SCORE_LUT_SIZE: usize = (0b1 << 8) + 1;
const VALUE_SCORE_LUT_SPAN_MASK: u32 = !(u32::MAX << 8);

// (open_fours(1), fours(2), open_threes(2), potential(3)
// overline override for full-bits
// total 8 bits
fn encode_value_key(pattern: Pattern) -> usize {
    let has_open_four = pattern.has_open_four() as u32;
    let total_fours = (pattern.count_total_fours() & 0b11) << 1;
    let open_threes = (pattern.count_open_threes() & 0b11) << 3;
    let potentials = (pattern.count_potentials() & 0b111) << 5;
    let has_overline = pattern.has_overline() as u32 * VALUE_SCORE_LUT_SPAN_MASK;

    (has_overline | has_open_four | total_fours | open_threes | potentials) as usize
}

type ValueScoreLut = [(i16, u16); VALUE_SCORE_LUT_SIZE];

const VALUE_SCORE_LUT: ColorContainer<ValueScoreLut> = build_value_score_lut();

const fn build_value_score_lut() -> ColorContainer<ValueScoreLut> {
    let mut acc = ColorContainer::new(
        [(0, 0); VALUE_SCORE_LUT_SIZE],
        [(0, 0); VALUE_SCORE_LUT_SIZE]
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
            let mut tactical_points = 0;

            match color {
                Color::Black => {
                    if pattern_key == VALUE_SCORE_LUT_SPAN_MASK as usize {
                        acc = HeuristicValueScores::OVERLINE_FORBID;
                    } else {
                        if fours > 1 {
                            acc = HeuristicValueScores::DOUBLE_FOUR_FORBID;
                        } else if open_threes > 1 {
                            acc = HeuristicValueScores::DOUBLE_THREE_FORBID;
                        } else if has_open_four {
                            acc = HeuristicValueScores::OPEN_FOUR;
                        } else if closed_fours == 1 && open_threes == 1 { // three-four fork
                            tactical_points += 1;
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
                    } else if closed_fours > 1 { // double-four fork
                        tactical_points += 1;
                    } else if closed_fours == 1 && open_threes > 0 { // three-four fork
                        tactical_points += 1;
                    } else if open_threes > 1 { // double-three fork
                        tactical_points += 1;
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

            lut[pattern_key] = (acc, tactical_points);
        });
    }

    flash_score_variants(Color::Black,  &mut acc.0[Color::Black as usize]);
    flash_score_variants(Color::White, &mut acc.0[Color::White as usize]);

    acc
}

struct HeuristicValueScores; impl HeuristicValueScores {
    const POTENTIAL: [i16; 5]       = [0, 1, 8, 16, 40];

    const OPEN_THREE: i16           = 256;
    const CLOSED_FOUR: i16          = 128;
    const OPEN_FOUR: i16            = 0;

    const OVERLINE_FORBID: i16      = -80;
    const DOUBLE_FOUR_FORBID: i16   = -50;
    const DOUBLE_THREE_FORBID: i16  = -40;
}

const POLICY_SCORE_LUT_SIZE: usize = (0b1 << 8) + 1;
const POLICY_SCORE_LUT_SPAN_MASK: u32 = !(u32::MAX << 8);

// (open_fours(1), fours(2), open_threes(2), potential(3)
// overline override for full-bits
// total 8 bits
fn encode_policy_key(pattern: Pattern) -> usize {
    let has_overline_pattern = (pattern.has_overline() as u32) * POLICY_SCORE_LUT_SPAN_MASK;
    let has_open_four = pattern.has_open_four() as u32;
    let total_fours = (pattern.count_total_fours() & 0b11) << 1;
    let open_threes = (pattern.count_open_threes() & 0b11) << 3;
    let potentials = (pattern.count_potentials() & 0b111) << 5;

    (has_overline_pattern | has_open_four | total_fours | open_threes | potentials) as usize
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
                    if pattern_key == POLICY_SCORE_LUT_SPAN_MASK as usize {
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

    flash_score_variants(Color::Black, &mut acc.0[Color::Black as usize]);
    flash_score_variants(Color::White, &mut acc.0[Color::White as usize]);

    acc
}

struct HeuristicPolicyScores; impl HeuristicPolicyScores {
    const POTENTIAL: [i8; 5] = [0, 1, 4, 12, 24];

    const OPEN_THREE: i8 = 92;
    const CLOSED_FOUR: i8 = 16;
    const OPEN_FOUR: i8 = i8::MAX;

    const DOUBLE_THREE_FORK: i8 = 70;
    const THREE_FOUR_FORK: i8 = i8::MAX;
    const DOUBLE_FOUR_FORK: i8 = i8::MAX;

    const DOUBLE_THREE_FORBID: i8 = i8::MIN;
    const DOUBLE_FOUR_FORBID: i8 = i8::MIN;
    const OVERLINE_FORBID: i8 = i8::MIN;
}
