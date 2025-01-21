use crate::history::History;
use crate::notation::pos;
use crate::notation::pos::Pos;
use rand::Rng;
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

pub fn find_forbidden_symmetry_moves(history: &History, fifth_move: Pos) -> HashSet<Pos> {
    let black_side_symmetry_moves = find_symmetry_moves(
        history.get(0).unwrap().unwrap(),
        history.get(2).unwrap().unwrap(),
        fifth_move
    );

    let white_side_symmetry_moves = find_symmetry_moves(
        history.get(1).unwrap().unwrap(),
        history.get(3).unwrap().unwrap(),
        fifth_move
    );

    black_side_symmetry_moves
        .intersection(&white_side_symmetry_moves)
        .filter(|pos| pos.idx() < pos::BOARD_SIZE as u8)
        .copied()
        .collect()
}

pub fn generate_random_opening_moves<const N: usize>() -> [Pos; N] {
    let mut raw_moves: [u8; N] = [0; N];
    raw_moves[0] = pos::CENTER.idx();

    fn calculate_abs_move(rel_move: u8, width: u8) -> u8 {
        const HALF: u8 = pos::BOARD_WIDTH / 2;
        let offset = HALF - width / 2;
        (rel_move / width + offset) * pos::BOARD_WIDTH + rel_move % width + offset
    }

    for idx in 1 .. N as u8 {
        let width = idx * 2 + 1;
        let mut rel_move = rand::thread_rng().gen_range(0 .. width * width - idx);

        for sub_idx in 0 .. idx as usize {
            if raw_moves[sub_idx] == calculate_abs_move(rel_move, width) {
                rel_move += 1;
            }
        }

        raw_moves[idx as usize] = calculate_abs_move(rel_move, width);
    }

    raw_moves.map(Pos::from_index)
}

pub fn generate_zeroing_moves() -> SmallVec<[Pos; 16]> {
    todo!()
}
