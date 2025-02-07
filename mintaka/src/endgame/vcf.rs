use crate::memo::transposition_table::TranspositionTable;
use crate::memo::tt_entry::{EndgameFlag, TTEntry, TTFlag};
use crate::thread_data::ThreadData;
use crate::value::{Depth, Eval, Score};
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::direction::Direction;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{Pos, BOARD_WIDTH};
use rusty_renju::pattern::{PatternCount, PatternUnit};
use smallvec::{smallvec, SmallVec};

trait VCFAccumulator {

    const COLD: Self;

    fn unit(pos: Pos) -> Self;

    fn append(self, defend: Pos, four: Pos) -> Self;

    fn has_result(&self) -> bool;

}

type SequenceVCFAccumulator = Option<Vec<Pos>>;

impl VCFAccumulator for SequenceVCFAccumulator {

    const COLD: Self = None;

    #[inline]
    fn unit(pos: Pos) -> Self {
        Some(vec![pos])
    }

    #[inline]
    fn append(self, defend: Pos, four: Pos) -> Self {
        self.map(|mut sequence| {
            sequence.push(defend);
            sequence.push(four);
            sequence
        })
    }

    #[inline]
    fn has_result(&self) -> bool {
        self.is_some()
    }

}

impl VCFAccumulator for Score {

    const COLD: Self = 0;

    #[inline]
    fn unit(_pos: Pos) -> Self {
        Score::MAX
    }

    #[inline]
    fn append(self, _defend: Pos, _four: Pos) -> Self {
        self
    }

    #[inline]
    fn has_result(&self) -> bool {
        *self != 0
    }

}

pub fn vcf<ACC: VCFAccumulator>(
    tt: &TranspositionTable, td: &mut ThreadData,
    board: &Board, max_depth: u8
) -> ACC {
    let mut board = board.clone();
    match board.player_color {
        Color::Black => try_vcf_flat::<{ Color::Black }, ACC>(tt, td, &board, max_depth, 0, false),
        Color::White => try_vcf_flat::<{ Color::White }, ACC>(tt, td, &board, max_depth, 0, false),
    }
}

pub fn vcf_sequence(
    tt: &TranspositionTable, td: &mut ThreadData,
    board: &Board, max_depth: u8
) -> Option<Vec<Pos>> {
    vcf::<SequenceVCFAccumulator>(tt, td, board, max_depth)
        .map(|mut sequence| {
            sequence.reverse();
            sequence
        })
}

fn try_vcf_flat<const C: Color, ACC: VCFAccumulator>(
    tt: &TranspositionTable, td: &mut ThreadData,
    source_board: &Board,
    max_depth: Depth, mut depth: Depth, mut opponent_has_five: bool,
) -> ACC {
    let mut idx: usize = 0;
    let mut board: Board = source_board.clone();

    // 2616 bytes
    #[derive(Copy, Clone)]
    #[repr(align(8))]
    struct SearchFrame {
        board: Board,
        next_idx: usize,
        depth: Depth,
        opponent_has_five: bool,
        four_pos: Pos,
        defend_pos: Pos,
    }

    // 62,784 bytes
    let mut stack: SmallVec<[SearchFrame; 18]> = smallvec![];

    #[inline]
    fn backtrace_frames<ACC: VCFAccumulator>(
        tt: &TranspositionTable, td: &mut ThreadData,
        board: Board,
        mut stack: SmallVec<[SearchFrame; 18]>, depth: Depth, four_pos: Pos
    ) -> ACC {
        let mut result = ACC::unit(four_pos);
        let mut hash_key = board.hash_key;

        let opponent_color = board.opponent_color();

        while !stack.is_empty() {
            let frame = stack.pop().unwrap();

            hash_key = hash_key.set(opponent_color, frame.defend_pos);
            tt.store_entry_mut(hash_key, build_vcf_lose_tt_entry(depth));

            hash_key = hash_key.set(board.player_color, frame.four_pos);
            tt.store_entry_mut(hash_key, build_vcf_win_tt_entry(depth, frame.four_pos));

            result = result.append(frame.defend_pos, frame.four_pos);
        }

        td.batch_counter.add_single_mut();

        result
    }

    'vcf_search: loop {
        'position_search: while idx < pos::BOARD_SIZE {
            let pattern = board.patterns.field[idx];
            let player_unit = pattern.player_unit::<C>();
            let opponent_unit = pattern.opponent_unit::<C>();

            if !player_unit.has_any_four()
                || (opponent_has_five && !opponent_unit.has_five())
                || (C == Color::Black && pattern.is_forbidden())
            {
                idx += 1;

                continue 'position_search;
            }

            let four_pos = Pos::from_index(idx as u8);

            if player_unit.has_open_four() {
                tt.store_entry_mut(board.hash_key, build_vcf_win_tt_entry(depth, four_pos));

                return backtrace_frames(tt, td, board, stack, depth, four_pos);
            }

            let mut position_board = board.set(four_pos);

            let defend_pos = find_defend_pos_unchecked::<C>(&position_board, four_pos, player_unit);
            let defend_pattern = position_board.patterns.field[defend_pos.idx_usize()];
            let defend_unit = defend_pattern.opponent_unit::<C>();
            let defend_four_count = defend_unit.count_fours();
            let defend_is_forbidden = C == Color::White && defend_pattern.is_forbidden();

            if match C {
                Color::Black => defend_four_count != PatternCount::Multiple
                    && !defend_unit.has_open_four(),
                Color::White => !defend_unit.has_open_four()
                    || defend_is_forbidden
            } {
                if match C {
                    Color::Black => defend_four_count == PatternCount::Cold
                        && player_unit.has_three(),
                    Color::White => defend_is_forbidden
                        || (defend_four_count == PatternCount::Cold
                        && player_unit.has_three())
                } {
                    tt.store_entry_mut(board.hash_key, build_vcf_win_tt_entry(depth, four_pos));

                    return backtrace_frames(tt, td, board, stack, depth, four_pos);
                } else if !tt.probe(
                    position_board.hash_key.set(C.reversed(), defend_pos)
                ).is_some_and(|entry| entry.endgame_flag == EndgameFlag::Cold) {
                    if depth + 2 > max_depth || position_board.stones + 3 >= pos::U8_BOARD_SIZE {
                        idx += 1;

                        continue 'position_search;
                    }

                    position_board.set_mut(defend_pos);

                    stack.push(SearchFrame {
                        board,
                        next_idx: idx + 1,
                        depth,
                        opponent_has_five,
                        four_pos,
                        defend_pos,
                    });

                    board = position_board;
                    idx = 0;
                    depth = depth + 2;
                    opponent_has_five = defend_four_count != PatternCount::Cold;

                    td.batch_counter.add_pair_mut();

                    continue 'vcf_search;
                }
            }

            td.batch_counter.add_single_mut();

            idx += 1;
        }

        let tt_entry = tt.probe(board.hash_key)
            .map(|mut tt_entry| {
                tt_entry.endgame_flag = EndgameFlag::Cold;
                tt_entry
            })
            .unwrap_or_else(|| TTEntry {
                best_move: Pos::INVALID,
                depth,
                flag: TTFlag::default(),
                endgame_flag: EndgameFlag::Cold,
                score: 0,
                eval: 0,
            });

        tt.store_entry_mut(board.hash_key, tt_entry);

        if let Some(frame) = stack.pop() {
            board = frame.board;
            idx = frame.next_idx;
            depth = frame.depth;
            opponent_has_five = frame.opponent_has_five;
        } else {
            break 'vcf_search;
        }
    }

    ACC::COLD
}

#[inline]
fn calculate_closed_four_window(pos: Pos, direction: Direction) -> (u8, u8) {
    fn saturating_sub_diff(n: u8) -> u8 {
        n - n.saturating_sub(4)
    }

    fn saturating_add_diff(n: u8) -> u8 {
        (n + 4).min(BOARD_WIDTH - 1) - n
    }

    let (negative_offset, positive_offset) = match direction {
        Direction::Horizontal => (
            saturating_sub_diff(pos.col()),
            saturating_add_diff(pos.col()),
        ),
        Direction::Vertical => (
            saturating_sub_diff(pos.row()),
            saturating_add_diff(pos.row())
        ),
        Direction::Ascending => (
            saturating_sub_diff(pos.row()).min(saturating_sub_diff(pos.col())),
            saturating_add_diff(pos.row()).min(saturating_add_diff(pos.col()))
        ),
        Direction::Descending => (
            saturating_add_diff(pos.row()).min(saturating_sub_diff(pos.col())),
            saturating_sub_diff(pos.row()).min(saturating_add_diff(pos.col()))
        )
    };

    (negative_offset, positive_offset)
}

#[inline]
fn find_defend_pos_unchecked<const C: Color>(board: &Board, four_pos: Pos, four_unit: PatternUnit) -> Pos {
    let four_direction = four_unit.closed_four_direction_unchecked();

    let (negative_offset, positive_offset) = calculate_closed_four_window(four_pos, four_direction);
    let start_pos = four_pos.offset_negative_unchecked(four_direction, negative_offset);

    for slice_offset in 0 ..= negative_offset + positive_offset {
        let pos = start_pos.offset_positive_unchecked(four_direction, slice_offset);

        if board.patterns.field[pos.idx_usize()].player_unit::<C>().has_five() {
            return pos;
        }
    }

    unreachable!()
}

#[inline]
fn build_vcf_win_tt_entry(depth: u8, four_pos: Pos) -> TTEntry {
    TTEntry {
        best_move: four_pos,
        depth,
        flag: TTFlag::Exact,
        endgame_flag: EndgameFlag::Win,
        score: Score::MAX,
        eval: Eval::MAX,
    }
}

#[inline]
fn build_vcf_lose_tt_entry(depth: u8) -> TTEntry {
   TTEntry {
       best_move: Pos::INVALID,
       depth,
       flag: TTFlag::Exact,
       endgame_flag: EndgameFlag::Lose,
       score: Score::MIN,
       eval: Eval::MIN,
   }
}
