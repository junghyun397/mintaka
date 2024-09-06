use crate::board_width;
use crate::check_cartesian_bound;
use crate::notation::history::History;
use crate::notation::pos::Pos;
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
    let black_symmetry_moves = find_symmetry_moves(
        history.get(0).unwrap(),
        history.get(2).unwrap(),
        fifth_move
    );

    let white_symmetry_moves = find_symmetry_moves(
        history.get(1).unwrap(),
        history.get(3).unwrap(),
        fifth_move
    );

    black_symmetry_moves
        .intersection(&white_symmetry_moves)
        .filter(|pos| check_cartesian_bound!(pos.row(), pos.col()))
        .copied()
        .collect()
}
