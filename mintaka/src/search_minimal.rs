use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::movegen::move_list::MoveEntry;
use crate::movegen::move_picker::MovePicker;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use crate::value::{Depth, MAX_PLY};
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::value::{Score, Scores};

pub fn iterative_deepening_minimal<const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<TH, impl Evaluator>,
    mut state: GameState,
) -> Score {
    let mut score: Score = 0;

    'iterative_deepening: for depth in 1 ..= td.config.max_depth {
        td.depth = depth;

        score = negamax::<R, TH>(td, &mut state, depth, -Score::INF, Score::INF);

        if td.is_aborted() {
            break 'iterative_deepening;
        }
    }

    score
}

fn negamax<const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<TH, impl Evaluator>,
    state: &mut GameState,
    depth_left: Depth,
    mut alpha: Score,
    beta: Score,
) -> Score {
    let is_root = td.ply == 0;

    if state.board.stones as usize >= td.config.draw_condition {
        return Score::DRAW;
    }

    if let &Some(pos) = state.board.patterns.unchecked_five_pos
        .access(state.board.player_color)
    { // immediate win
        if is_root {
            td.best_move = pos.into();
        }

        return Score::win_in(td.ply + 1)
    }

    if let &Some(pos) = state.board.patterns.unchecked_five_pos
        .access(!state.board.player_color)
    { // defend immediate win
        if state.board.player_color == Color::Black
            && state.board.patterns.forbidden_field.is_hot(pos)
        { // trapped
            if is_root {
                td.best_move = MaybePos::NONE;
            }

            return Score::lose_in(td.ply + 2)
        }

        td.ss[td.ply].movegen_window = state.movegen_window;
        td.push_ply_mut(pos);
        state.set_mut(pos);

        let score = -negamax::<R, TH>(td, state, depth_left.saturating_sub(1), -beta, -alpha);

        td.pop_ply_mut();
        state.unset_mut(td.ss[td.ply].movegen_window);

        if is_root {
            td.best_move = pos.into();
        }

        return score;
    }

    let static_eval = td.evaluator.eval_value(state);

    if td.ply + 1 >= MAX_PLY || depth_left == 0 {
        return static_eval;
    }

    if td.should_check_limit()
        && td.search_limit_exceeded()
    {
        td.set_aborted_mut();
        return Score::DRAW;
    }

    if td.is_aborted() {
        return Score::DRAW;
    }

    let mut best_score = -Score::INF;
    let mut best_move = MaybePos::NONE;

    let mut move_picker = MovePicker::new(MaybePos::NONE, [MaybePos::NONE; 2]);

    let mut moves_made = 0;
    'position_search: while let Some(MoveEntry { pos, .. }) = move_picker.next(td, state) {
        if !state.board.is_legal_move(pos) {
            continue;
        }

        td.push_ply_mut(pos);
        state.set_mut(pos);
        td.ss[td.ply].last_pos = pos.into();

        moves_made += 1;

        let score = -negamax::<R, TH>(td, state, depth_left - 1, -beta, -alpha);

        td.pop_ply_mut();
        state.unset_mut(td.ss[td.ply].movegen_window);

        if is_root { // todo: debug
            td.root_moves += 1;
            td.root_scores[pos.idx_usize()] = score as f32;
        }

        if score <= best_score {
            continue;
        }

        best_score = score;
        best_move = pos.into();

        alpha = alpha.max(score);

        if alpha >= beta {
            break 'position_search;
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
