use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use rusty_renju::board::{Board, MoveArtifact};
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::{ForbiddenKind, RuleKind};
use rusty_renju::notation::score::Score;
use rusty_renju::pattern::Pattern;
use rusty_renju::slice::Slices;
use rusty_renju::utils::empty::Empty;
use rusty_renju::{const_for, pattern};

const BLACK_SIGNUM: ColorContainer<i32> = ColorContainer::new(1, -1);

#[derive(Clone)]
pub struct HeuristicEvaluator<const R: RuleKind> {
    scores: ColorContainer<[i16; pattern::PATTERN_SIZE]>,
    ordering_scores: [ColorContainer<i16>; pattern::PATTERN_SIZE],
    score_black: i32,
    hash_key: HashKey,
}

impl<const R: RuleKind> HeuristicEvaluator<R> {
    fn update(&mut self, board: &Board<R>, artifact: MoveArtifact, plied: Pos) {
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
                    let key = encode_key(board.patterns.field[color][pos.idx_usize()]);

                    let score = VALUE_SCORE_LUT[key];
                    let old_score = std::mem::replace(&mut self.scores[color][pos.idx_usize()], score);
                    let pattern_delta = score as i32 - old_score as i32;

                    let ordering_score = ORDERING_SCORE_LUT[key];
                    self.ordering_scores[pos.idx_usize()][color] = ordering_score;

                    score_delta += pattern_delta;
                }
            }

            self.score_black += score_delta * BLACK_SIGNUM[color];
        }
    }
}

impl<const R: RuleKind> Evaluator<R> for HeuristicEvaluator<R> {
    type EvaluatorParameter = ();

    fn require_stabilize(&self) -> bool {
        true
    }

    fn from_state(state: &GameState<R>) -> Self {
        let mut evaluator = Self {
            scores: ColorContainer::new([0; pattern::PATTERN_SIZE], [0; pattern::PATTERN_SIZE]),
            ordering_scores: [ColorContainer::new(0, 0); pattern::PATTERN_SIZE],
            score_black: 0,
            hash_key: HashKey::empty(),
        };

        evaluator.init(&state.board);

        evaluator
    }

    fn init(&mut self, board: &Board<R>) {
        self.hash_key = board.hash_key;

        for color in [Color::Black, Color::White] {
            for idx in 0 .. pos::BOARD_SIZE {
                let key = encode_key(board.patterns.field[color][idx]);
                let pattern_score = VALUE_SCORE_LUT[key];

                self.scores[color][idx] = pattern_score;
                self.ordering_scores[idx][color] = ORDERING_SCORE_LUT[key];
            }
        }

        self.score_black = self.scores[Color::Black].iter().map(|&score| score as i32).sum::<i32>()
            - self.scores[Color::White].iter().map(|&score| score as i32).sum::<i32>();
    }

    fn play(&mut self, board: &Board<R>, artifact: MoveArtifact, plied: MaybePos) {
        if let Some(plied) = plied.ok() {
            self.update(board, artifact, plied);
        }

        self.hash_key = board.hash_key;
    }

    fn undo(&mut self, board: &Board<R>, artifact: MoveArtifact, removed: MaybePos) {
        if let Some(removed) = removed.ok() {
            self.update(board, artifact, removed);
        }

        self.hash_key = board.hash_key;
    }

    fn eval_policy(&mut self, _: &GameState<R>) {}

    fn eval_value(&mut self, state: &GameState<R>) -> Score {
        let mut score_black = self.score_black;

        if R == RuleKind::Renju {
            let forbidden_field = state.board.patterns.forbidden_field & !state.board.hot_field;

            for pos in forbidden_field.iter_hot_pos() {
                let kind = state.board.patterns.forbidden_kind(pos).unwrap();

                score_black += forbidden_penalty::<EvaluationScores>(kind) as i32
                    - self.scores[Color::Black][pos.idx_usize()] as i32;
            }
        }

        let max_score = Score::MATE_MIN.value() - 1;

        Score::from_i32((score_black * BLACK_SIGNUM[state.board.player_color]).clamp(-max_score, max_score))
    }

    fn ordering_score(&self, board: &Board<R>, pos: Pos) -> i16 {
        let mut score = self.ordering_scores[pos.idx_usize()];

        if let Some(kind) = board.patterns.forbidden_kind(pos) {
            if board.player_color == Color::Black {
                return forbidden_penalty::<OrderingScores>(kind);
            }

            score[Color::Black] = 0;
        }

        ((score[board.player_color] as i32 * 5 + score[!board.player_color] as i32 * 3) / 3) as i16
    }

    fn hash_key(&self) -> HashKey {
        self.hash_key
    }
}

fn forbidden_penalty<W: WeightSet>(kind: ForbiddenKind) -> i16 {
    match kind {
        ForbiddenKind::Overline => W::OVERLINE_PENALTY,
        ForbiddenKind::DoubleFour => W::DOUBLE_FOUR_PENALTY,
        ForbiddenKind::DoubleThree => W::DOUBLE_THREE_PENALTY,
    }
}

fn encode_key(pattern: Pattern) -> usize {
    let mut acc = 0;

    acc |= (pattern.count_closed_fours().min(3) as usize) << 6;
    acc |= (pattern.count_open_threes().min(3) as usize) << 4;
    acc |= (pattern.count_potential_four().min(3) as usize) << 2;
    acc |= pattern.count_potential_three().min(3) as usize;

    acc
}

const SCORE_LUT_SIZE: usize = u8::MAX as usize + 1;

type ScoreLut = [i16; SCORE_LUT_SIZE];

const VALUE_SCORE_LUT: ScoreLut = build_score_lut::<EvaluationScores>();
const ORDERING_SCORE_LUT: ScoreLut = build_score_lut::<OrderingScores>();

const fn build_score_lut<W: WeightSet>() -> ScoreLut {
    let mut lut = [0; SCORE_LUT_SIZE];

    const_for!(pattern_key in 0, SCORE_LUT_SIZE; {
        let closed_fours = pattern_key >> 6;
        let open_threes = (pattern_key >> 4) & 0b11;
        let potential_fours = (pattern_key >> 2) & 0b11;
        let potential_threes = pattern_key & 0b11;

        lut[pattern_key] = if closed_fours > 1 { // double-four fork
            W::DOUBLE_FOUR_FORK
        } else if closed_fours == 1 && open_threes > 0 { // three-four fork
            W::THREE_FOUR_FORK
        } else if open_threes > 1 { // double-three fork
            W::DOUBLE_THREE_FORK
        } else {
            let primary_index = if closed_fours != 0 {
                2
            } else if open_threes != 0 {
                1
            } else {
                0
            };

            W::MAIN_TABLE[primary_index][potential_threes][potential_fours]
        }
    });

    lut
}

trait WeightSet {
    // MAIN_TABLE[none | three | four][potential-threes][potential-fours]
    const MAIN_TABLE: [[[i16; 4]; 4]; 3];

    const DOUBLE_FOUR_FORK: i16;
    const THREE_FOUR_FORK: i16;
    const DOUBLE_THREE_FORK: i16;

    const OVERLINE_PENALTY: i16;
    const DOUBLE_FOUR_PENALTY: i16;
    const DOUBLE_THREE_PENALTY: i16;
}

struct OrderingScores;

// baseline closed-four = 100
impl WeightSet for OrderingScores {
    const MAIN_TABLE: [[[i16; 4]; 4]; 3] = [
        // none
        [
            [0, 30, 60, 90],
            [20, 50, 80, 110],
            [40, 70, 100, 130],
            [60, 90, 120, 150],
        ],
        // open-three
        [
            [60, 90, 120, 150],
            [80, 110, 140, 170],
            [100, 130, 160, 190],
            [120, 150, 180, 210],
        ],
        // closed-four
        [
            [100, 130, 160, 190],
            [120, 150, 180, 210],
            [140, 170, 200, 230],
            [160, 190, 220, 250],
        ],
    ];

    const DOUBLE_FOUR_FORK: i16 = 1000;
    const THREE_FOUR_FORK: i16 = 600;
    const DOUBLE_THREE_FORK: i16 = 200;

    const OVERLINE_PENALTY: i16      = -400;
    const DOUBLE_FOUR_PENALTY: i16   = -300;
    const DOUBLE_THREE_PENALTY: i16  = -150;
}

struct EvaluationScores;

// baseline closed-four = 100
impl WeightSet for EvaluationScores {
    const MAIN_TABLE: [[[i16; 4]; 4]; 3] = [
        // none
        [
            [0, 13, 29, 48],
            [7, 22, 40, 61],
            [16, 33, 53, 76],
            [27, 46, 68, 93],
        ],
        // open-three
        [
            [60, 78, 99, 123],
            [72, 92, 115, 141],
            [86, 108, 133, 161],
            [102, 126, 153, 183],
        ],
        // closed-four
        [
            [100, 120, 143, 169],
            [111, 133, 158, 186],
            [124, 148, 175, 205],
            [139, 165, 194, 226],
        ],
    ];

    const DOUBLE_FOUR_FORK: i16 = 1000;
    const THREE_FOUR_FORK: i16 = 600;
    const DOUBLE_THREE_FORK: i16 = 180;

    const OVERLINE_PENALTY: i16      = -400;
    const DOUBLE_FOUR_PENALTY: i16   = -300;
    const DOUBLE_THREE_PENALTY: i16  = -150;
}
