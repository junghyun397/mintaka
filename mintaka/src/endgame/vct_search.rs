use crate::endgame::accumulator::{EndgameAccumulator, SequenceEndgameAccumulator};
use crate::memo::tt_entry::TTEntry;
use crate::movegen::move_list::MoveList;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::Score;

pub(crate) struct VCTFrame {
    vct_moves: MoveList,
    next_move_counter: usize,
    depth: usize,
    opponent_has_open_four: bool,
    threat_pos: Pos,
    defend_pos: Pos,
}

pub fn vct_search(
    td: &mut ThreadData<impl ThreadType>,
    board: &Board, max_depth: usize
) -> Score {
    vct::<Score>(td, board, max_depth)
}

pub fn vct_sequence(
    td: &mut ThreadData<impl ThreadType>,
    board: &Board, max_depth: usize
) -> Option<Vec<Pos>> {
    vct::<SequenceEndgameAccumulator>(td, board, max_depth)
        .map(|mut sequence| {
            sequence.reverse();
            sequence
        })
}

fn vct<ACC: EndgameAccumulator>(
    td: &mut ThreadData<impl ThreadType>,
    board: &Board, max_depth: usize
) -> ACC {
    let mut board = *board;
    match board.player_color {
        Color::Black => try_vct::<{ Color::Black }, ACC>(td, board, max_depth, 0, false, false),
        Color::White => try_vct::<{ Color::White }, ACC>(td, board, max_depth, 0, false, false),
    }
}

// depth-first proof-number search
fn try_vct<const C: Color, ACC: EndgameAccumulator>(
    td: &mut ThreadData<impl ThreadType>,
    mut board: Board,
    max_depth: usize, mut depth: usize, mut opponent_has_open_four: bool, mut opponent_has_five: bool,
) -> ACC {
    // TODO: implement

    ACC::ZERO
}

#[inline]
fn find_defend_open_four_unchecked<const C: Color>(board: &Board) -> Pos {
    todo!()
}

#[inline]
fn find_vcf_to_defend_pos<const C: Color>(board: &Board) -> MaybePos {
    todo!()
}

#[inline]
fn build_vct_win_tt_entry(depth: usize, four_pos: Pos) -> TTEntry {
    todo!()
}

#[inline]
fn build_vcf_lose_tt_entry(depth: usize) -> TTEntry {
    todo!()
}
