use crate::eval::evaluator::Evaluator;
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

pub const TT_MOVE_SCORE: i16 = Score::MATE.value() as i16 - 300;
pub const DIRECT_RESPONSE_SCORE: i16 = Score::INF.value() as i16 - 500;
pub const KILLER_MOVE_SCORE: i16 = Score::INF.value() as i16 - 1000;
pub const COUNTER_MOVE_BONUS: i16 = 100;

const ENDGAME_MOVEGEN_IMPRINT_MASK_LUT: [Bitfield; pos::BOARD_SIZE] = build_imprint_mask_lut([
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

pub fn generate_endgame_moves<const R: RuleKind, const T: ThreatSearchKind>(
    td: &ThreadData<R, impl ThreadType, impl Evaluator<R>>,
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
        moves.push(EndgameMoveEntry {
            pos,
            score: td.evaluator.ordering_score(&state.board, pos),
        });
    }

    moves
}

pub fn generate_threat_direct_response<const R: RuleKind>(
    buffer: &mut MainMoveList,
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    state: &GameState<R>,
    field: &Bitfield,
) {
    for pos in field.iter_hot_pos() {
        let mut score = DIRECT_RESPONSE_SCORE;

        // threat score
        if state.board.patterns.field[state.board.player_color][pos.idx_usize()].has_any_threat() {
            score += 100;
        }

        buffer.push(MainMoveEntry { pos, score, lp_quiet: false, history_score: None });
    }
}

pub fn generate_extend_four_response<const R: RuleKind>(
    buffer: &mut MainMoveList,
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    state: &GameState<R>,
) {
    let maybe_last_pos = state.history.last_action_or_none();

    let counter_move = counter_move_from(td, state)
        .filter(|pos| state.board.patterns.field[state.board.player_color][pos.idx_usize()].has_closed_four());

    for pos in state.board.patterns.indexes[state.board.player_color].closed_fours.iter_hot_pos() {
        let mut score = 0;

        // distance score
        if let Some(last_pos) = maybe_last_pos.ok() {
            score += (15 - pos.distance(last_pos)) as i16;
        }

        // counter-move score
        if let Some(counter_move) = counter_move && counter_move == pos {
            score += COUNTER_MOVE_BONUS;
        }

        // history score
        score += td.ht.four[state.board.player_color][pos.idx_usize()] / 128;

        buffer.push(MainMoveEntry { pos, score, lp_quiet: false, history_score: None });
    }
}

pub fn generate_all_moves<const R: RuleKind>(
    buffer: &mut MainMoveList,
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    state: &GameState<R>,
) {
    td.evaluator.eval_policy(state);

    let field = state.board.legal_field(state.board.player_color) & state.movegen_window.movegen_field;
    let player_pattern = &state.board.patterns.field[state.board.player_color];

    let counter_move = counter_move_from(td, state);

    for pos in field.iter_hot_pos() {
        let idx = pos.idx_usize();
        let player_pattern = player_pattern[idx];

        // policy score
        let mut score = td.evaluator.ordering_score(&state.board, pos);

        // counter-move score
        if let Some(counter_move) = counter_move && pos == counter_move {
            score += COUNTER_MOVE_BONUS;
        }

        // history score
        let history_score;
        if player_pattern.has_open_three() {
            history_score = td.ht.three[state.board.player_color][idx];

            score += history_score / 256;
        } else if player_pattern.has_any_four() {
            history_score = td.ht.four[state.board.player_color][idx];

            score += history_score / 256;
        } else {
            history_score = td.ht.quiet[state.board.player_color][idx];

            score += history_score / 512;
        };

        buffer.push(MainMoveEntry { pos, score, lp_quiet: false, history_score: Some(history_score) });
    }
}

fn counter_move_from<const R: RuleKind>(
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    state: &GameState<R>,
) -> Option<Pos> {
    state.history.last_action()
        .and_then(|action| action.ok())
        .and_then(|last_pos|
            td.ht.counter[state.board.player_color][last_pos.idx_usize()].ok()
        )
}
