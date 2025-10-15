use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::movegen::move_list::MoveEntry;
use crate::movegen::move_picker::MovePicker;
use crate::protocol::response::Response;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use crate::value;
use crate::value::Depth;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{Score, Scores};

pub fn iterative_deepening_minimal<const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<TH, impl Evaluator>,
    mut state: GameState,
) -> (Score, MaybePos) {
    if !TH::IS_MAIN { // enforce single-thread
        return (0, MaybePos::NONE);
    }

    let mut score: Score = 0;
    let mut best_move = MaybePos::NONE;

    'iterative_deepening: for depth in 1 ..= td.config.max_depth {
        td.depth = depth;

        let iter_score = pvs_minimal::<R, TH>(td, &mut state, true, depth, -Score::INF, Score::INF);

        if td.is_aborted() {
            break 'iterative_deepening;
        }

        score = iter_score;
        best_move = td.best_move;
        td.depth_reached = depth;

        if TH::IS_MAIN {
            td.thread_type.make_response(Response::Status {
                best_move,
                score,
                pv: td.pvs[0],
                total_nodes_in_1k: td.batch_counter.count_global_in_1k(),
                depth
            })
        }
    }

    (score, best_move)
}

pub fn pvs_minimal<const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<TH, impl Evaluator>,
    state: &mut GameState,
    on_pv: bool,
    depth_left: Depth,
    mut alpha: Score,
    mut beta: Score,
) -> Score {
    let is_root = td.ply == 0;

    if TH::IS_MAIN
        && td.should_check_limit()
        && td.search_limit_exceeded()
    {
        td.set_aborted_mut();
        return Score::DRAW;
    }

    if td.is_aborted() || state.board.stones as usize >= td.config.draw_condition {
        return Score::DRAW;
    }

    td.pvs[td.ply].clear();

    // immediate win
    if let &Some(pos) = state.board.patterns.unchecked_five_pos.access(state.board.player_color) {
        if is_root {
            td.best_move = pos.into();
        }

        if on_pv {
            td.pvs[td.ply].init(pos.into());
        }

        return Score::win_in(td.ply + 1)
    }

    // defend immediate win
    if let &Some(pos) = state.board.patterns.unchecked_five_pos.access(!state.board.player_color) {
        if state.board.player_color == Color::Black
            && state.board.patterns.forbidden_field.is_hot(pos)
        { // trapped
            if is_root {
                td.best_move = MaybePos::NONE;
            }

            if on_pv {
                td.pvs[td.ply].init(MaybePos::NONE);
            }

            return Score::lose_in(td.ply + 2)
        }

        td.ss[td.ply].recovery_state = state.recovery_state();
        td.push_ply_mut(pos);
        state.set_mut(pos);

        let score = -pvs_minimal::<R, TH>(td, state, on_pv, depth_left, -beta, -alpha);

        td.pop_ply_mut();
        state.undo_mut(td.ss[td.ply].recovery_state);

        if td.is_aborted() {
            return Score::DRAW;
        }

        if is_root {
            td.best_move = pos.into();
        }

        if on_pv {
            let sub_pv = td.pvs[td.ply + 1];
            td.pvs[td.ply].load(pos.into(), sub_pv);
        }

        return score;
    }

    if !is_root {
        alpha = alpha.max(Score::lose_in(td.ply));
        beta = beta.min(Score::win_in(td.ply));
        if alpha >= beta { // mate distance pruning
            return alpha;
        }
    }

    let static_eval = td.evaluator.eval_value(state);

    if depth_left <= 0 || td.ply >= value::MAX_PLY {
        // return vcf_search::<R>(td, td.config.max_vcf_depth, state, alpha, beta, static_eval);
        return static_eval;
    }

    let mut best_score = -Score::INF;
    let mut best_move = MaybePos::NONE;

    let mut move_picker = MovePicker::new(MaybePos::NONE, [MaybePos::NONE; 2]);
    let mut moves_made = 0;

    'position_search: while let Some(MoveEntry { pos, .. }) = move_picker.next(td, state) {
        if !state.board.is_legal_move(pos) {
            continue;
        }

        td.ss[td.ply].recovery_state = state.recovery_state();
        td.push_ply_mut(pos);
        state.set_mut(pos);

        moves_made += 1;

        let mut extension = 0;

        // extend on three-four-fork, open-four, double-three-fork, open-four, double-four-fork
        if state.board.patterns.counts.global.access(state.board.player_color).open_fours != 0 {
            extension += 1;
        }

        let score = if moves_made == 1 {
            -pvs_minimal::<R, TH>(td, state, on_pv, depth_left + extension - 1, -beta, -alpha)
        } else { // zero-window search
            let mut score = -pvs_minimal::<R, TH>(td, state, false, depth_left + extension - 1, -alpha - 1, -alpha);

            if score > alpha { // zero-window failed, full-window re-search on PV
                score = -pvs_minimal::<R, TH>(td, state, on_pv, depth_left + extension - 1, -beta, -alpha);
            }

            score
        };

        td.pop_ply_mut();
        state.undo_mut(td.ss[td.ply].recovery_state);

        if td.is_aborted() {
            return Score::DRAW;
        }

        if score <= best_score {
            continue;
        }

        best_score = score;

        if score > alpha { // improve alpha
            best_move = pos.into();
            alpha = score;

            if on_pv {
                let sub_pv = td.pvs[td.ply + 1];
                td.pvs[td.ply].load(pos.into(), sub_pv);
            }

            if alpha >= beta { // beta cutoff
                break 'position_search;
            }
        }
    }

    if is_root {
        td.best_move = best_move;
    }

    if moves_made == 0 {
        static_eval
    } else {
        best_score
    }
}
