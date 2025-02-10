use crate::endgame::accumulator::{EndgameAccumulator, SequenceEndgameAccumulator};
use crate::memo::transposition_table::TranspositionTable;
use crate::memo::tt_entry::TTEntry;
use crate::thread_data::ThreadData;
use crate::value::{Depth, Score};
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use smallvec::{smallvec, SmallVec};

pub(crate) struct VCTFrame {
    board: Board,
    next_idx: usize,
    depth: Depth,
    opponent_has_open_four: bool,
    opponent_has_five: bool,
    threat_pos: Pos,
    defend_pos: Pos,
}

pub fn vct_search(
    tt: &TranspositionTable, td: &mut ThreadData,
    board: &Board, max_depth: Depth
) -> Score {
    vct::<Score>(tt, td, board, max_depth)
}

pub fn vct_sequence(
    tt: &TranspositionTable, td: &mut ThreadData,
    board: &Board, max_depth: Depth
) -> Option<Vec<Pos>> {
    vct::<SequenceEndgameAccumulator>(tt, td, board, max_depth)
        .map(|mut sequence| {
            sequence.reverse();
            sequence
        })
}

fn vct<ACC: EndgameAccumulator>(
    tt: &TranspositionTable, td: &mut ThreadData,
    board: &Board, max_depth: Depth
) -> ACC {
    let mut board = *board;
    match board.player_color {
        Color::Black => try_vct::<{ Color::Black }, ACC>(tt, td, board, max_depth, 0, false, false),
        Color::White => try_vct::<{ Color::White }, ACC>(tt, td, board, max_depth, 0, false, false),
    }
}

// depth-first proof-number search
fn try_vct<const C: Color, ACC: EndgameAccumulator>(
    tt: &TranspositionTable, td: &mut ThreadData,
    mut board: Board,
    max_depth: Depth, mut depth: Depth, mut opponent_has_open_four: bool, mut opponent_has_five: bool,
) -> ACC {
    let mut idx: usize = 0;

    let mut stack: SmallVec<[VCTFrame; 32]> = smallvec![];

    #[inline]
    fn backtrace_frames<ACC: EndgameAccumulator>(
        tt: &TranspositionTable, td: &mut ThreadData, mut stack: SmallVec<[VCTFrame; 32]>,
        board: Board, depth: Depth, killer_pos: Pos
    ) -> ACC {
        let mut result = ACC::unit(killer_pos);
        let mut hash_key = board.hash_key;

        let opponent_color = board.opponent_color();

        while let Some(frame) = stack.pop() {
            hash_key = hash_key.set(opponent_color, frame.defend_pos);
            tt.store_entry_mut(hash_key, build_vcf_lose_tt_entry(depth));

            hash_key = hash_key.set(board.player_color, frame.threat_pos);
            tt.store_entry_mut(hash_key, build_vct_win_tt_entry(depth, frame.threat_pos));

            result = result.append(frame.defend_pos, frame.threat_pos);
        }

        td.batch_counter.add_single_mut();

        result
    }

    'vct_search: loop {
        'position_search: while idx < pos::BOARD_SIZE {
            idx += 1;
        }
    }

    ACC::COLD
}

#[inline]
fn find_defend_open_four_unchecked<const C: Color>(board: &Board) -> Pos {
    todo!()
}

#[inline]
fn find_vcf_to_defend_pos<const C: Color>(board: &Board) -> Option<Pos> {
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
