use crate::notation::pos;
use crate::notation::pos::{MaybePos, Pos};
use rand::{rng, Rng};
use smallvec::SmallVec;
use std::collections::HashSet;

fn find_symmetry_moves(ref1: Pos, ref2: Pos, m: Pos) -> HashSet<Pos> {
    if ref1.row() == ref2.row() || ref1.col() == ref2.col() {
        let reversed_row = ref1.row() + ref2.row() - m.row();
        let reversed_col = ref1.col() + ref2.col() - m.col();

        // . . | . .
        // . M | X .
        // __1_|_2__
        // . X | X .
        // . . | . .
        HashSet::from([
            Pos::from_cartesian(reversed_row, reversed_col),
            Pos::from_cartesian(m.row(), reversed_col),
            Pos::from_cartesian(reversed_row, m.col())
        ])
    } else {
        // y=ax+b
        // a=(y1-y2)/(x1-x2)
        let slope = (ref1.row() - ref2.row()) as f64 / (ref1.col() - ref2.col()) as f64;

        // b=y-ax
        let intercept = ref1.row() as f64 - slope * ref1.col() as f64;

        // 2(ax-y+b)/(a^2+1)
        let base_eval = 2.0 * (slope * m.col() as f64 - m.row() as f64 + intercept) / (slope.powi(2) + 1.0);

        // x'=x-2a(ax-y+b)/(a^2+1)
        let reversed_col = (m.col() as f64 - slope * base_eval) as u8;

        // y'=y+2(ax-y+b)/(a^2+1)
        let reversed_row = m.row() + base_eval as u8;

        // . M . . .
        // X 1 . . .
        // . . . 2 X
        // . . . X .
        HashSet::from([
            Pos::from_cartesian(reversed_row, reversed_col),
            Pos::from_cartesian(ref1.row() + ref2.row() - reversed_row, ref1.col() + ref2.col() - reversed_col),
            Pos::from_cartesian(ref1.row() + ref2.row() - m.row(), ref1.col() + ref2.col() - m.col())
        ])
    }
}

pub fn find_forbidden_symmetry_moves(history: &[Pos; 4], fifth_move: Pos) -> SmallVec<[Pos; 3]> {
    let black_side_symmetry_moves = find_symmetry_moves(history[0], history[1], fifth_move);

    let white_side_symmetry_moves = find_symmetry_moves(history[2], history[3], fifth_move);

    black_side_symmetry_moves
        .intersection(&white_side_symmetry_moves)
        .filter(|pos| pos.idx() < pos::BOARD_SIZE as u8)
        .copied()
        .collect()
}

pub fn generate_random_opening_moves<const N: usize>() -> [Pos; N] {
    let mut raw_moves: [u8; N] = [MaybePos::NONE.unwrap().idx(); N];
    raw_moves[0] = pos::CENTER.idx();

    fn generate_move_in(width: u8) -> u8 {
        let rel_move = rng().random_range(0 .. width * width);
        let offset = pos::BOARD_WIDTH / 2 - width / 2;
        (rel_move / width + offset) * pos::BOARD_WIDTH + rel_move % width + offset
    }

    for idx in 1 .. N as u8 {
        let width = idx * 2 + 1;
        let mut abs_move = generate_move_in(width);

        while raw_moves[.. idx as usize].contains(&abs_move) {
            abs_move = generate_move_in(width);
        }

        raw_moves[idx as usize] = abs_move;
    }

    raw_moves.map(Pos::from_index)
}
