use crate::game_state::GameState;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos};
use rusty_renju::notation::score::{Score, Scores};

pub fn find_immediate_win(state: &GameState, ply: usize) -> (Score, MaybePos) {
    if let Some(pos) = state.board.patterns.unchecked_five_pos[state.board.player_color].ok()
    { // five
        return (Score::win_in(ply + 1), pos.into())
    }

    if let Some(pos) = state.board.patterns.unchecked_five_pos[!state.board.player_color].ok() {
        if state.board.player_color == Color::Black
            && state.board.patterns.is_forbidden(pos)
        { // trap
            return (Score::lose_in(ply + 2), MaybePos::NONE)
        }

        if 1 < state.board.patterns.field[!state.board.player_color].iter()
            .filter(|pattern| pattern.has_five())
            .count()
        { // opponent-five
            return (Score::lose_in(ply + 2), pos.into())
        }

        return (Score::NAN, MaybePos::NONE)
    }

    if state.board.patterns.counts.global[state.board.player_color].open_fours > 0
        && let Some(pos) = state.board.legal_field(state.board.player_color).iter_hot_pos()
            .find(|pos|
                state.board.patterns.field[state.board.player_color][pos.idx_usize()].has_open_four()
            )
    { // open-four
        return (Score::win_in(ply + 3), pos.into());
    }

    (Score::NAN, MaybePos::NONE)
}
