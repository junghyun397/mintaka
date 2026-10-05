use crate::game_state::GameState;
use rusty_renju::board::{Board, MoveArtifact};
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::direction::Direction;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::Score;
use rusty_renju::pattern::Pattern;
use rusty_renju::slice::{self, Slice, Slices};
use rusty_renju::utils::empty::Empty;
use rusty_renju::{const_for, pattern};

const PATTERN_SCORE_DIVISOR: i32 = 4;
const TEMPO_BONUS: i32 = 115;
const COUNTER_ATTACK_BONUS: i32 = 33;
const FORBIDDEN_ORDERING_SCORE: i16 = -1024;
const MAX_ORDERING_SCORE: i32 = 30000;

#[derive(Clone)]
pub struct HeuristicEvaluator<const R: RuleKind> {
    pattern_scores: [ColorContainer<PatternScore>; pattern::PATTERN_SIZE],
    pattern_score_black: i32,
    line_scores: LineScores,
    hash_key: HashKey,
}

impl<const R: RuleKind> HeuristicEvaluator<R> {
    pub fn from_state(state: &GameState<R>) -> Self {
        let mut evaluator = Self {
            pattern_scores: [ColorContainer::splat(PatternScore::ZERO); pattern::PATTERN_SIZE],
            pattern_score_black: 0,
            line_scores: LineScores::EMPTY,
            hash_key: HashKey::empty(),
        };

        evaluator.init(&state.board);

        evaluator
    }

    pub fn init(&mut self, board: &Board<R>) {
        self.hash_key = board.hash_key;
        self.pattern_score_black = 0;
        self.line_scores.init::<R>(&board.slices);

        for idx in 0 .. pos::BOARD_SIZE {
            for color in [Color::Black, Color::White] {
                self.pattern_scores[idx][color] = PatternScore::from_pattern(board.patterns.field[color][idx]);
            }

            self.pattern_score_black += self.pattern_scores[idx][Color::Black].value as i32
                - self.pattern_scores[idx][Color::White].value as i32;
        }
    }

    pub fn play(&mut self, board: &Board<R>, artifact: MoveArtifact, plied: MaybePos) {
        if let Some(plied) = plied.ok() {
            self.update(board, artifact, plied);
        }

        self.hash_key = board.hash_key;
    }

    pub fn undo(&mut self, board: &Board<R>, artifact: MoveArtifact, removed: MaybePos) {
        if let Some(removed) = removed.ok() {
            self.update(board, artifact, removed);
        }

        self.hash_key = board.hash_key;
    }

    pub fn require_stabilize(&self) -> bool {
        false
    }

    pub fn eval_value(&mut self, state: &GameState<R>) -> Score {
        let sign = if state.board.player_color == Color::Black { 1 } else { -1 };

        let pattern_score = sign * self.legal_pattern_score_black(&state.board) / PATTERN_SCORE_DIVISOR;
        let line_score = sign * self.line_scores.score_black();
        let own_pressure = threat_pressure(&state.board, state.board.player_color);
        let opponent_pressure = threat_pressure(&state.board, !state.board.player_color);
        let pressure_score = (own_pressure * 2 - opponent_pressure) / 2;
        let counter_attack_bonus = if can_counter_attack(&state.board) { COUNTER_ATTACK_BONUS } else { 0 };

        let score = pattern_score + line_score + pressure_score + TEMPO_BONUS + counter_attack_bonus;
        let max_score = Score::MATE_MIN.value_i32() - 1;

        Score::from_i32(score.clamp(-max_score, max_score))
    }

    pub fn ordering_score(&self, board: &Board<R>, pos: Pos) -> i16 {
        let mut scores = self.pattern_scores[pos.idx_usize()];

        if board.patterns.is_forbidden(pos) {
            if board.player_color == Color::Black {
                return FORBIDDEN_ORDERING_SCORE;
            }

            scores[Color::Black] = PatternScore::ZERO;
        }

        let attack = scores[board.player_color].attack_score() as i32;
        let defense = scores[!board.player_color].defense_score() as i32;

        (attack + defense).min(MAX_ORDERING_SCORE) as i16
    }

    pub fn hash_key(&self) -> HashKey {
        self.hash_key
    }

    fn legal_pattern_score_black(&self, board: &Board<R>) -> i32 {
        let mut score = self.pattern_score_black;

        if R == RuleKind::Renju {
            let forbidden_field = board.patterns.forbidden_field & !board.hot_field;

            for pos in forbidden_field.iter_hot_pos() {
                score -= self.pattern_scores[pos.idx_usize()][Color::Black].value as i32;
            }
        }

        score
    }

    fn update(&mut self, board: &Board<R>, artifact: MoveArtifact, plied: Pos) {
        let black_delta = self.update_color::<{ Color::Black }>(board, &artifact, plied);
        let white_delta = self.update_color::<{ Color::White }>(board, &artifact, plied);

        self.pattern_score_black += black_delta - white_delta;
        self.line_scores.update::<R>(&board.slices, plied);
    }

    #[inline(always)]
    fn update_color<const C: Color>(&mut self, board: &Board<R>, artifact: &MoveArtifact, plied: Pos) -> i32 {
        self.update_direction::<C, { Direction::Horizontal }>(
            board, artifact[C][Direction::Horizontal], plied,
        ) + self.update_direction::<C, { Direction::Vertical }>(
            board, artifact[C][Direction::Vertical], plied,
        ) + self.update_direction::<C, { Direction::Ascending }>(
            board, artifact[C][Direction::Ascending], plied,
        ) + self.update_direction::<C, { Direction::Descending }>(
            board, artifact[C][Direction::Descending], plied,
        )
    }

    #[inline(always)]
    fn update_direction<const C: Color, const D: Direction>(
        &mut self, board: &Board<R>, mut changed_bitmap: u16, plied: Pos,
    ) -> i32 {
        let start_pos = Slices::slice_start_pos(D, plied);
        let mut score_delta = 0;

        while changed_bitmap != 0 {
            let slice_idx = changed_bitmap.trailing_zeros() as usize;
            changed_bitmap &= changed_bitmap - 1;

            let pos = start_pos.directional_offset_unchecked(D, slice_idx as isize);
            let score = PatternScore::from_pattern(board.patterns.field[C][pos.idx_usize()]);
            let old_score = std::mem::replace(&mut self.pattern_scores[pos.idx_usize()][C], score);

            score_delta += score.value as i32 - old_score.value as i32;
        }

        score_delta
    }
}

struct ThreatPressureWeights {
    open_three: i32,
    closed_four: i32,
    close_three: i32,
}

fn threat_pressure<const R: RuleKind>(board: &Board<R>, color: Color) -> i32 {
    const COUNT_WEIGHT: [i32; 5] = [0, 16, 24, 28, 30];
    const COUNT_WEIGHT_SCALE: i32 = 16;

    const ATTACK: ThreatPressureWeights = ThreatPressureWeights {
        open_three: 115,
        closed_four: 27,
        close_three: 0,
    };

    const DEFENSE: ThreatPressureWeights = ThreatPressureWeights {
        open_three: 87,
        closed_four: 0,
        close_three: 247,
    };

    let legal = board.legal_field(color);
    let indexes = &board.patterns.indexes[color];
    let open_threes = (indexes.open_threes & legal).count_hots().min(4) as usize;
    let closed_fours = (indexes.closed_fours & legal).count_hots().min(4) as usize;
    let close_threes = (indexes.close_threes & legal).count_hots().min(4) as usize;
    let weights = if color == board.player_color { ATTACK } else { DEFENSE };

    (weights.open_three * COUNT_WEIGHT[open_threes]
        + weights.closed_four * COUNT_WEIGHT[closed_fours]
        + weights.close_three * COUNT_WEIGHT[close_threes]) / COUNT_WEIGHT_SCALE
}

fn can_counter_attack<const R: RuleKind>(board: &Board<R>) -> bool {
    let player = board.player_color;

    !board.patterns.effective_fork_four_field(!player).is_empty()
        && !(board.patterns.indexes[player].closed_fours & board.legal_field(player)).is_empty()
}

const WINDOW_LENGTH: u8 = 5;
const WINDOW_MASK: u16 = (1 << WINDOW_LENGTH) - 1;
const WINDOW_LUT_SIZE: usize = 1 << (WINDOW_LENGTH * 2);
const MAX_LINE_WINDOWS: usize = pos::U_BOARD_WIDTH - WINDOW_LENGTH as usize + 1;

#[derive(Clone)]
struct LineScores {
    window_scores: [[i16; MAX_LINE_WINDOWS]; slice::TOTAL_SLICE_AMOUNT],
    score_black: i32,
}

impl LineScores {
    const EMPTY: Self = Self {
        window_scores: [[0; MAX_LINE_WINDOWS]; slice::TOTAL_SLICE_AMOUNT],
        score_black: 0,
    };

    fn init<const R: RuleKind>(&mut self, slices: &Slices) {
        self.window_scores.fill([0; MAX_LINE_WINDOWS]);
        self.score_black = 0;

        for line in &slices.horizontal_slices {
            self.update_windows::<R>(line, Direction::Horizontal, 0, line.length - WINDOW_LENGTH);
        }

        for line in &slices.vertical_slices {
            self.update_windows::<R>(line, Direction::Vertical, 0, line.length - WINDOW_LENGTH);
        }

        for line in &slices.ascending_slices {
            self.update_windows::<R>(line, Direction::Ascending, 0, line.length - WINDOW_LENGTH);
        }

        for line in &slices.descending_slices {
            self.update_windows::<R>(line, Direction::Descending, 0, line.length - WINDOW_LENGTH);
        }
    }

    fn update<const R: RuleKind>(&mut self, slices: &Slices, pos: Pos) {
        self.update_line_at::<R>(&slices.horizontal_slices[pos.row_usize()], Direction::Horizontal, pos.col());
        self.update_line_at::<R>(&slices.vertical_slices[pos.col_usize()], Direction::Vertical, pos.row());

        if let Some(line) = slices.ascending_slice(pos) {
            self.update_line_at::<R>(line, Direction::Ascending, pos.col() - line.start_col);
        }
        if let Some(line) = slices.descending_slice(pos) {
            self.update_line_at::<R>(line, Direction::Descending, pos.col() - line.start_col);
        }
    }

    fn score_black(&self) -> i32 {
        self.score_black
    }

    fn update_line_at<const R: RuleKind>(&mut self, line: &Slice, direction: Direction, index: u8) {
        // overline
        let first = index.saturating_sub(WINDOW_LENGTH);
        let last = (index + 1).min(line.length - WINDOW_LENGTH);

        self.update_windows::<R>(line, direction, first, last);
    }

    fn update_windows<const R: RuleKind>(&mut self, line: &Slice, direction: Direction, first: u8, last: u8) {
        let line_idx = direction_offset(direction) + line.idx as usize;

        for start in first ..= last {
            let score = window_value::<R>(line, start);
            let old_score = std::mem::replace(&mut self.window_scores[line_idx][start as usize], score);

            self.score_black += score as i32 - old_score as i32;
        }
    }
}

fn direction_offset(direction: Direction) -> usize {
    match direction {
        Direction::Horizontal => 0,
        Direction::Vertical => pos::U_BOARD_WIDTH,
        Direction::Ascending => pos::U_BOARD_WIDTH * 2,
        Direction::Descending => pos::U_BOARD_WIDTH * 2 + slice::DIAGONAL_SLICE_AMOUNT,
    }
}

static WINDOW_LUT: [i16; WINDOW_LUT_SIZE] = build_window_lut();

const fn build_window_lut() -> [i16; WINDOW_LUT_SIZE] {
    const STONE_VALUE: [i16; WINDOW_LENGTH as usize + 1] = [0, 3, 15, 42, 160, 2400];
    let mut lut = [0; WINDOW_LUT_SIZE];

    const_for!(key in 0, WINDOW_LUT_SIZE; {
        let black = key & WINDOW_MASK as usize;
        let white = key >> WINDOW_LENGTH;

        lut[key] = if white == 0 {
            STONE_VALUE[black.count_ones() as usize]
        } else if black == 0 {
            -STONE_VALUE[white.count_ones() as usize]
        } else {
            0
        };
    });

    lut
}

fn window_value<const R: RuleKind>(line: &Slice, start: u8) -> i16 {
    let black = line.stones[Color::Black];
    let white = line.stones[Color::White];
    let black_window = (black >> start) & WINDOW_MASK;
    let white_window = (white >> start) & WINDOW_MASK;
    let key = black_window | (white_window << WINDOW_LENGTH);
    let score = WINDOW_LUT[key as usize];

    let before_window = (1u32 << start) >> 1;
    let after_window = 1u32 << (start + WINDOW_LENGTH);
    let neighbors = before_window | after_window;
    let black_window_invalid = R != RuleKind::Freestyle && score > 0 && black as u32 & neighbors != 0;
    let white_window_invalid = R == RuleKind::Gomoku && score < 0 && white as u32 & neighbors != 0;

    if black_window_invalid || white_window_invalid {
        0
    } else {
        score
    }
}

const SUPPORT_COUNT_SIZE: usize = 4;
const MAX_SUPPORT_COUNT: u32 = SUPPORT_COUNT_SIZE as u32 - 1;
const SUPPORT_LUT_SIZE: usize = SUPPORT_COUNT_SIZE * SUPPORT_COUNT_SIZE * SUPPORT_COUNT_SIZE;
const SCORE_LUT_SIZE: usize = Threat::ALL.len() * SUPPORT_LUT_SIZE;

#[derive(Copy, Clone)]
struct PatternScore {
    value: i16,
    key: u16,
}

impl PatternScore {
    const ZERO: Self = Self { value: 0, key: 0 };

    fn from_pattern(pattern: Pattern) -> Self {
        SCORE_LUT[pattern_key(pattern)]
    }

    fn attack_score(self) -> i16 {
        ATTACK_LUT[self.key as usize]
    }

    fn defense_score(self) -> i16 {
        DEFENSE_LUT[self.key as usize]
    }
}

#[derive(Copy, Clone)]
enum Threat {
    Quiet,
    OpenThree,
    DoubleThree,
    ClosedFour,
    ThreeFour,
    ForkFour,
    Five,
}

impl Threat {
    const ALL: [Self; 7] = [
        Self::Quiet, Self::OpenThree, Self::DoubleThree, Self::ClosedFour,
        Self::ThreeFour, Self::ForkFour, Self::Five,
    ];

    fn from_pattern(pattern: Pattern) -> Self {
        let closed_fours = pattern.count_closed_four();
        let open_threes = pattern.count_open_three();

        if pattern.has_five() {
            Self::Five
        } else if pattern.has_open_four() || closed_fours > 1 {
            Self::ForkFour
        } else if closed_fours != 0 && open_threes != 0 {
            Self::ThreeFour
        } else if closed_fours != 0 {
            Self::ClosedFour
        } else if open_threes > 1 {
            Self::DoubleThree
        } else if open_threes != 0 {
            Self::OpenThree
        } else {
            Self::Quiet
        }
    }
}

fn pattern_key(pattern: Pattern) -> usize {
    if pattern.is_empty() {
        return 0;
    }

    let threat = Threat::from_pattern(pattern);
    let close_threes = pattern.count_close_three().min(MAX_SUPPORT_COUNT) as usize;
    let potential_fours = pattern.count_potential_four().min(MAX_SUPPORT_COUNT) as usize;
    let potential_threes = pattern.count_potential_three().min(MAX_SUPPORT_COUNT) as usize;

    score_key(threat, close_threes, potential_fours, potential_threes)
}

const fn score_key(threat: Threat, close_threes: usize, potential_fours: usize, potential_threes: usize) -> usize {
    ((threat as usize * SUPPORT_COUNT_SIZE + close_threes) * SUPPORT_COUNT_SIZE + potential_fours)
        * SUPPORT_COUNT_SIZE + potential_threes
}

static SCORE_LUT: [PatternScore; SCORE_LUT_SIZE] = build_score_lut();

const fn build_score_lut() -> [PatternScore; SCORE_LUT_SIZE] {
    let mut lut = [PatternScore::ZERO; SCORE_LUT_SIZE];

    const_for!(threat_idx in 0, Threat::ALL.len(); {
        let threat = Threat::ALL[threat_idx];
        let values = point_values(threat);

        const_for!(close_threes in 0, SUPPORT_COUNT_SIZE; {
            const_for!(potential_fours in 0, SUPPORT_COUNT_SIZE; {
                const_for!(potential_threes in 0, SUPPORT_COUNT_SIZE; {
                    let key = score_key(threat, close_threes, potential_fours, potential_threes);

                    lut[key] = PatternScore {
                        value: values[close_threes][potential_fours][potential_threes],
                        key: key as u16,
                    };
                });
            });
        });
    });

    lut
}

struct PolicyWeights {
    closed_four: i16,
    open_three: i16,
    double_three: i16,
    three_four: i16,
    fork_four: i16,
    five: i16,
    close_three: i16,
    potential_four: i16,
    potential_three: i16,
    close_three_potential_four: i16,
    support_potential_three: i16,
    close_three_pairs: i16,
    potential_four_pairs: i16,
    potential_three_pairs: i16,
}

impl PolicyWeights {
    const fn score(&self, threat: Threat, close_threes: i16, potential_fours: i16, potential_threes: i16) -> i16 {
        let forcing = match threat {
            Threat::Quiet => 0,
            Threat::OpenThree => self.open_three,
            Threat::DoubleThree => self.double_three,
            Threat::ClosedFour => self.closed_four,
            Threat::ThreeFour => self.three_four,
            Threat::ForkFour => self.fork_four,
            Threat::Five => self.five,
        };

        forcing
            + self.close_three * close_threes
            + self.potential_four * potential_fours
            + self.potential_three * potential_threes
            + self.close_three_potential_four * close_threes * potential_fours
            + self.support_potential_three * (close_threes + potential_fours) * potential_threes
            + self.close_three_pairs * close_threes * (close_threes - 1)
            + self.potential_four_pairs * potential_fours * (potential_fours - 1)
            + self.potential_three_pairs * potential_threes * (potential_threes - 1)
    }
}

const ATTACK_WEIGHTS: PolicyWeights = PolicyWeights {
    closed_four: 203,
    open_three: 270,
    double_three: 519,
    three_four: 676,
    fork_four: 2000,
    five: 4000,
    close_three: 0,
    potential_four: 115,
    potential_three: 104,
    close_three_potential_four: 0,
    support_potential_three: 0,
    close_three_pairs: 17,
    potential_four_pairs: 6,
    potential_three_pairs: 0,
};

const DEFENSE_WEIGHTS: PolicyWeights = PolicyWeights {
    closed_four: 192,
    open_three: 124,
    double_three: 149,
    three_four: 700,
    fork_four: 484,
    five: 2400,
    close_three: 0,
    potential_four: 47,
    potential_three: 22,
    close_three_potential_four: 10,
    support_potential_three: 0,
    close_three_pairs: 10,
    potential_four_pairs: 4,
    potential_three_pairs: 0,
};

static ATTACK_LUT: [i16; SCORE_LUT_SIZE] = build_policy_lut(ATTACK_WEIGHTS);
static DEFENSE_LUT: [i16; SCORE_LUT_SIZE] = build_policy_lut(DEFENSE_WEIGHTS);

const fn build_policy_lut(weights: PolicyWeights) -> [i16; SCORE_LUT_SIZE] {
    let mut lut = [0; SCORE_LUT_SIZE];

    const_for!(threat_idx in 0, Threat::ALL.len(); {
        let threat = Threat::ALL[threat_idx];

        const_for!(close_threes in 0, SUPPORT_COUNT_SIZE; {
            const_for!(potential_fours in 0, SUPPORT_COUNT_SIZE; {
                const_for!(potential_threes in 0, SUPPORT_COUNT_SIZE; {
                    let key = score_key(threat, close_threes, potential_fours, potential_threes);

                    lut[key] = weights.score(threat, close_threes as i16, potential_fours as i16, potential_threes as i16);
                });
            });
        });
    });

    lut
}

// [close threes, 3][potential fours, 3][potential threes, 3]
type SupportValues = [[[i16; SUPPORT_COUNT_SIZE]; SUPPORT_COUNT_SIZE]; SUPPORT_COUNT_SIZE];

const fn point_values(threat: Threat) -> SupportValues {
    match threat {
        Threat::Quiet => [
            [[0, 0, 1, 0], [0, 58, 76, 0], [62, 80, 0, 382], [163, 469, 528, 660]],
            [[64, 108, 152, 196], [102, 190, 278, 366], [198, 330, 462, 594], [352, 528, 704, 880]],
            [[128, 216, 304, 392], [174, 306, 438, 570], [278, 454, 630, 806], [440, 660, 880, 1100]],
            [[192, 324, 456, 588], [246, 422, 598, 774], [358, 578, 798, 1018], [528, 792, 1056, 1320]],
        ],
        Threat::OpenThree => [
            [[5, 30, 54, 223], [161, 126, 56, 209], [8, 272, 356, 444], [320, 473, 605, 737]],
            [[128, 172, 216, 260], [181, 269, 357, 445], [292, 424, 556, 688], [461, 637, 813, 989]],
            [[224, 312, 400, 488], [285, 417, 549, 681], [404, 580, 756, 932], [581, 801, 1021, 1241]],
            [[320, 452, 584, 716], [389, 565, 741, 917], [516, 736, 956, 1176], [701, 965, 1229, 1493]],
        ],
        Threat::DoubleThree => [
            [[0, 0, 284, 0], [68, 0, 133, 177], [148, 236, 324, 412], [309, 441, 573, 705]],
            [[96, 140, 184, 228], [149, 237, 325, 413], [260, 392, 524, 656], [429, 605, 781, 957]],
            [[192, 280, 368, 456], [253, 385, 517, 649], [372, 548, 724, 900], [549, 769, 989, 1209]],
            [[288, 420, 552, 684], [357, 533, 709, 885], [484, 704, 924, 1144], [669, 933, 1197, 1461]],
        ],
        Threat::ClosedFour => [
            [[100, 403, 539, 714], [321, 706, 860, 277], [694, 649, 424, 512], [486, 541, 673, 805]],
            [[222, 83, 0, 328], [126, 174, 425, 513], [270, 492, 624, 756], [529, 705, 881, 1057]],
            [[292, 380, 468, 556], [353, 485, 617, 749], [472, 648, 824, 1000], [649, 869, 1089, 1309]],
            [[388, 520, 652, 784], [457, 633, 809, 985], [584, 804, 1024, 1244], [769, 1033, 1297, 1561]],
        ],
        Threat::ThreeFour => [
            [[1457, 1600, 1658, 1493], [1667, 1608, 1626, 1670], [1641, 1729, 1817, 1905], [1802, 1934, 2066, 2198]],
            [[0, 507, 1677, 1721], [686, 1730, 1818, 1906], [1753, 1885, 2017, 2149], [1922, 2098, 2274, 2450]],
            [[1685, 1773, 1861, 1949], [1746, 1878, 2010, 2142], [1865, 2041, 2217, 2393], [2042, 2262, 2482, 2702]],
            [[1781, 1913, 2045, 2177], [1850, 2026, 2202, 2378], [1977, 2197, 2417, 2637], [2162, 2426, 2690, 2954]],
        ],
        Threat::ForkFour => [
            [[822, 177, 387, 387], [471, 476, 520, 564], [535, 623, 711, 799], [696, 828, 960, 1092]],
            [[259, 215, 155, 491], [338, 123, 296, 800], [278, 779, 911, 1043], [816, 992, 1168, 1344]],
            [[0, 667, 755, 843], [640, 772, 904, 1036], [759, 935, 1111, 1287], [936, 1156, 1376, 1596]],
            [[675, 807, 939, 1071], [744, 920, 1096, 1272], [871, 1091, 1311, 1531], [1056, 1320, 1584, 1848]],
        ],
        Threat::Five => [
            [[1600, 1600, 1600, 1600], [1645, 1689, 1733, 1777], [1748, 1836, 1924, 2012], [1909, 2041, 2173, 2305]],
            [[1696, 1740, 1784, 1828], [1749, 1837, 1925, 2013], [1860, 1992, 2124, 2256], [2029, 2205, 2381, 2557]],
            [[1792, 1880, 1968, 2056], [1853, 1985, 2117, 2249], [1972, 2148, 2324, 2500], [2149, 2369, 2589, 2809]],
            [[1888, 2020, 2152, 2284], [1957, 2133, 2309, 2485], [2084, 2304, 2524, 2744], [2269, 2533, 2797, 3061]],
        ],
    }
}
