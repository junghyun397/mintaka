use crate::game_state::GameState;
use crate::movegen::move_list::{EndgameMoveEntry, EndgameMoveList, MainMoveEntry, MainMoveList};
use crate::search_endgame::ThreatSearchKind;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::bitfield::{Bitfield, build_imprint_mask_lut};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::Score;
use rusty_renju::utils::empty::Empty;

pub const TT_MOVE_SCORE: i16 = Score::MATE.value_i32() as i16 - 300;
pub const KILLER_MOVE_SCORE: i16 = Score::INF.value_i32() as i16 - 1000;
pub const COUNTER_MOVE_BONUS: i16 = 100;

const TACTICAL_HISTORY_DIVISOR: i16 = 256;
const RESPONSE_HISTORY_DIVISOR: i16 = 128;
const QUIET_HISTORY_DIVISOR: i16 = 512;

static ENDGAME_MOVEGEN_IMPRINT_MASK_LUT: [Bitfield; pos::BOARD_SIZE] = build_imprint_mask_lut([
    0b100010001,
    0b010010010,
    0b001111100,
    0b001111100,
    0b111101111,
    0b001111100,
    0b001111100,
    0b010010010,
    0b100010001,
]);

pub fn generate_full_endgame_moves<const R: RuleKind, const T: ThreatSearchKind>(
    state: &GameState<R>,
) -> EndgameMoveList {
    let mut moves = EndgameMoveList::empty();

    let indexes = &state.board.patterns.indexes[state.board.player_color];
    let mut field = indexes.closed_fours | indexes.fork_fours;

    if T == ThreatSearchKind::VCT {
        field |= indexes.open_threes;
    }

    for pos in field.iter_hot_pos() {
        moves.push(EndgameMoveEntry { pos, score: 0 });
    }

    moves
}

pub fn generate_endgame_moves<const R: RuleKind, const T: ThreatSearchKind, const FILTER: u8>(
    td: &ThreadData<R, impl ThreadType>,
    state: &GameState<R>,
    recent_four: Pos,
) -> EndgameMoveList {
    let mut moves = EndgameMoveList::empty();

    let indexes = &state.board.patterns.indexes[state.board.player_color];
    let mut field = indexes.closed_fours | indexes.fork_fours;

    if T == ThreatSearchKind::VCT {
        field |= indexes.open_threes;
    }

    field &= ENDGAME_MOVEGEN_IMPRINT_MASK_LUT[recent_four.idx_usize()];

    for pos in field.iter_hot_pos() {
        if FILTER != 0
            && !state.board.patterns.field[state.board.player_color][pos.idx_usize()].has::<FILTER>()
        {
            continue;
        }

        moves.push(EndgameMoveEntry {
            pos,
            score: td.evaluator.ordering_score(&state.board, pos),
        });
    }

    moves
}

pub fn generate_moves<const R: RuleKind, const ON_THREAT: bool>(
    buffer: &mut MainMoveList,
    td: &mut ThreadData<R, impl ThreadType>,
    state: &GameState<R>,
    field: Bitfield,
) {
    let color = state.board.player_color;
    let player_patterns = &state.board.patterns.field[color];
    let opponent_patterns = &state.board.patterns.field[!color];

    let counter_move = counter_move_from(td, state);
    let tactical_history_divisor = if ON_THREAT {
        RESPONSE_HISTORY_DIVISOR
    } else {
        TACTICAL_HISTORY_DIVISOR
    };

    for pos in field.iter_hot_pos() {
        let idx = pos.idx_usize();
        let player_pattern = player_patterns[idx];

        let use_history = !ON_THREAT || player_pattern.has_closed_four();
        let history_score = if !use_history {
            0
        } else if player_pattern.has_open_three() {
            td.ht.three[color][idx]
        } else if player_pattern.has_any_four() {
            td.ht.four[color][idx]
        } else {
            td.ht.quiet[color][idx]
        };

        let history_bonus = if player_pattern.is_tactical() {
            history_score / tactical_history_divisor
        } else {
            history_score / QUIET_HISTORY_DIVISOR
        };

        let mut score = td.evaluator.ordering_score(&state.board, pos)
            + history_bonus;

        if use_history && counter_move == Some(pos) {
            score += COUNTER_MOVE_BONUS;
        }

        let lp_quiet = !ON_THREAT
            && !player_pattern.is_tactical()
            && !opponent_patterns[idx].has_open_three();

        buffer.push(MainMoveEntry {
            pos,
            score,
            history_score: Some(history_score),
            lp_quiet,
        });
    }
}

fn counter_move_from<const R: RuleKind>(
    td: &mut ThreadData<R, impl ThreadType>,
    state: &GameState<R>,
) -> Option<Pos> {
    state.history.last_action()
        .and_then(|action| action.ok())
        .and_then(|last_pos|
            td.ht.counter[state.board.player_color][last_pos.idx_usize()].ok()
        )
}
