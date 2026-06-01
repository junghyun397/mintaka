use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::game_state::GameState;
use crate::movegen::neighbor_scores::NeighborScores;
use rusty_renju::board::{Board, MoveArtifact};
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::ForbiddenKind;
use rusty_renju::notation::score::Score;
use rusty_renju::pattern::Pattern;
use rusty_renju::slice::Slices;
use rusty_renju::utils::empty::Empty;
use rusty_renju::{const_for, pattern};
use rusty_renju::notation::pos;

#[derive(Clone)]
pub struct HeuristicEvaluator {
    neighbor_scores: NeighborScores,
    scores: ColorContainer<[i16; pattern::PATTERN_SIZE]>,
    score_black: Score,
    hash_key: HashKey,
}

impl HeuristicEvaluator {
    fn update(&mut self, board: &Board, artifact: MoveArtifact, plied: Pos) {
        for (color, directions) in artifact.iter() {
            let mut score_delta = 0;

            for (direction, &changed_bitmap) in directions.iter() {
                if changed_bitmap == 0 {
                    continue;
                }

                let start_pos = Slices::slice_start_pos(direction, plied);

                let mut changed_bitmap = changed_bitmap;
                while changed_bitmap != 0 {
                    let slice_idx = changed_bitmap.trailing_zeros() as usize;
                    changed_bitmap &= changed_bitmap - 1;

                    let pos = start_pos.directional_offset_unchecked(direction, slice_idx as isize);
                    let key = encode_value_key(board.patterns.field[color][pos.idx_usize()]);

                    let score = PATTERN_SCORE_LUT[key];
                    let old_score = std::mem::replace(&mut self.scores[color][pos.idx_usize()], score);

                    score_delta += (score - old_score) as Score;
                }
            }

            self.score_black += score_delta * BLACK_SIGNUM[color];
        }
    }
}

impl Evaluator for HeuristicEvaluator {
    type EvaluatorParameter = ();

    fn require_stabilize(&self) -> bool {
        true
    }

    fn from_state(state: &GameState) -> Self {
        let mut evaluator = Self {
            neighbor_scores: NeighborScores::EMPTY,
            scores: ColorContainer::new([0; pattern::PATTERN_SIZE], [0; pattern::PATTERN_SIZE]),
            score_black: 0,
            hash_key: HashKey::empty(),
        };

        evaluator.init(&state.board);

        evaluator
    }

    fn init(&mut self, board: &Board) {
        self.neighbor_scores = (&board.hot_field).into();
        self.hash_key = board.hash_key;

        for idx in 0 .. pos::BOARD_SIZE {
            for color in [Color::Black, Color::White] {
                let key = encode_value_key(board.patterns.field[color][idx]);
                self.scores[color][idx] = PATTERN_SCORE_LUT[key];
            }
        }

        self.score_black =
             self.scores[Color::Black].iter().map(|&score| score as Score).sum::<Score>()
                - self.scores[Color::White].iter().map(|&score| score as Score).sum::<Score>();
    }

    fn play(&mut self, board: &Board, artifact: MoveArtifact, plied: Pos) {
        self.neighbor_scores.add_neighbor_score(plied);
        self.update(&board, artifact, plied);
        self.hash_key = board.hash_key;
    }

    fn undo(&mut self, board: &Board, artifact: MoveArtifact, removed: Pos) {
        self.neighbor_scores.remove_neighbor_score(removed);
        self.update(&board, artifact, removed);
        self.hash_key = board.hash_key;
    }

    fn eval_policy(&mut self, state: &GameState) -> PolicyDistribution {
        self.scores[state.board.player_color]
    }

    fn eval_value(&mut self, state: &GameState) -> Score {
        let mut forbidden_score = 0;
        for pos in state.board.patterns.forbidden_field.iter_hot_pos() {
            forbidden_score += match state.board.patterns.forbidden_kind(pos).unwrap() {
                ForbiddenKind::Overline => HeuristicPatternScores::OVERLINE_FORBID,
                ForbiddenKind::DoubleFour => HeuristicPatternScores::DOUBLE_FOUR_FORBID,
                ForbiddenKind::DoubleThree => HeuristicPatternScores::DOUBLE_THREE_FORBID,
            } as Score
        }

        (self.score_black - forbidden_score)
            * BLACK_SIGNUM[state.board.player_color]
    }

    fn hash_key(&self) -> HashKey {
        self.hash_key
    }
}

// open_fours(1), fours(2), open_threes(2), potential(3) 8 bits
fn encode_value_key(pattern: Pattern) -> usize {
    let has_open_four = pattern.has_open_four() as u32;
    let total_fours = (pattern.count_closed_fours() & 0b11) << 1;
    let open_threes = (pattern.count_open_threes() & 0b11) << 3;
    let potentials = (pattern.count_any_potential() & 0b111) << 5;

    (has_open_four | total_fours | open_threes | potentials) as usize
}

const VALUE_SCORE_LUT_SIZE: usize = (0b1 << 8) + 1;

type ValueScoreLut = [i16; VALUE_SCORE_LUT_SIZE];

const PATTERN_SCORE_LUT: ValueScoreLut = build_pattern_score_lut();

const fn build_pattern_score_lut() -> ValueScoreLut {
    let mut acc = [0; VALUE_SCORE_LUT_SIZE];

    const fn flash_score_variants(lut: &mut ValueScoreLut) {
        const_for!(pattern_key in 0, VALUE_SCORE_LUT_SIZE; {
            let has_open_four = (pattern_key & 0b1) == 0b1;
            let closed_fours = (pattern_key >> 1) & 0b11;
            let open_threes = (pattern_key >> 3) & 0b11;
            let potentials = (pattern_key >> 5) & 0b111;

            let mut acc = 0;

            if has_open_four {
                acc = HeuristicPatternScores::OPEN_FOUR;
            } else if closed_fours > 1 { // double-four fork
                acc = HeuristicPatternScores::DOUBLE_FOUR_FORK;
            } else if closed_fours == 1 && open_threes > 0 { // three-four fork
                acc = HeuristicPatternScores::THREE_FOUR_FORK;
            } else if open_threes > 1 { // double-three fork
                acc = HeuristicPatternScores::DOUBLE_THREE_FORK;
            }

            acc += open_threes as i16 * HeuristicPatternScores::OPEN_THREE;
            acc += closed_fours as i16 * HeuristicPatternScores::CLOSED_FOUR;
            acc += HeuristicPatternScores::POTENTIAL[potentials];

            lut[pattern_key] = acc;
        });
    }

    flash_score_variants(&mut acc);

    acc
}

struct HeuristicPatternScores;

impl HeuristicPatternScores {
    const POTENTIAL: [i16; 8]       = [0, 4, 12, 24, 40, 60, 84, 112];

    const CLOSED_FOUR: i16          = 300;
    const OPEN_THREE: i16           = 160;
    const OPEN_FOUR: i16            = 1000;

    const THREE_FOUR_FORK: i16      = 800 - Self::DOUBLE_THREE_FORK - Self::OPEN_FOUR;
    const DOUBLE_THREE_FORK: i16  = 300 - Self::OPEN_THREE * 2;
    const DOUBLE_FOUR_FORK: i16   = 1000 - Self::CLOSED_FOUR * 2;

    const OVERLINE_FORBID: i16      = 400;
    const DOUBLE_FOUR_FORBID: i16  = Self::DOUBLE_FOUR_FORK + 200;
    const DOUBLE_THREE_FORBID: i16 = Self::DOUBLE_THREE_FORK + 50;
}

const BLACK_SIGNUM: ColorContainer<Score> = ColorContainer::new(1, -1);
