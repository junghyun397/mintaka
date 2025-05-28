use crate::endgame::accumulator::{EndgameAccumulator, SequenceEndgameAccumulator};
use crate::memo::tt_entry::TTEntry;
use crate::movegen::move_list::MoveList;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use arrayvec::ArrayVec;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::{Depth, Score};

pub(crate) struct VCTFrame {
    vct_moves: MoveList,
    next_move_counter: usize,
    depth: Depth,
    opponent_has_open_four: bool,
    threat_pos: Pos,
    defend_pos: Pos,
}

pub fn vct_search(
    td: &mut ThreadData<impl ThreadType>,
    board: &Board, max_depth: Depth
) -> Score {
    vct::<Score>(td, board, max_depth)
}

pub fn vct_sequence(
    td: &mut ThreadData<impl ThreadType>,
    board: &Board, max_depth: Depth
) -> Option<Vec<Pos>> {
    vct::<SequenceEndgameAccumulator>(td, board, max_depth)
        .map(|mut sequence| {
            sequence.reverse();
            sequence
        })
}

fn vct<ACC: EndgameAccumulator>(
    td: &mut ThreadData<impl ThreadType>,
    board: &Board, max_depth: Depth
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
    max_depth: Depth, mut depth: Depth, mut opponent_has_open_four: bool, mut opponent_has_five: bool,
) -> ACC {
    let mut idx: usize = 0;

    #[inline]
    fn backtrace_frames<ACC: EndgameAccumulator>(
        td: &mut ThreadData<impl ThreadType>, mut stack: ArrayVec<VCTFrame, 32>,
        board: Board, depth: Depth, killer_pos: Pos
    ) -> ACC {
        let mut result = ACC::unit(killer_pos, 0);
        let mut hash_key = board.hash_key;

        let opponent_color = board.opponent_color();

        while let Some(frame) = stack.pop() {
            hash_key = hash_key.set(opponent_color, frame.defend_pos);
            td.tt.store_entry_mut(hash_key, build_vcf_lose_tt_entry(depth));

            hash_key = hash_key.set(board.player_color, frame.threat_pos);
            td.tt.store_entry_mut(hash_key, build_vct_win_tt_entry(depth, frame.threat_pos));

            result = result.append_pos(frame.defend_pos, frame.threat_pos);
        }

        td.batch_counter.add_single_mut();

        result
    }

    'vct_search: loop {
        'position_search: while idx < pos::BOARD_SIZE {
            idx += 1;
        }
    }

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
fn build_vct_win_tt_entry(depth: Depth, four_pos: Pos) -> TTEntry {
    todo!()
}

#[inline]
fn build_vcf_lose_tt_entry(depth: Depth) -> TTEntry {
    todo!()
}
