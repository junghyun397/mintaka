#![feature(test)]

extern crate test;

mod bench_forbidden_moves {
    use indoc::indoc;
    use rusty_renju::board::Board;
    use rusty_renju::notation::pos::Pos;
    use test::Bencher;

    #[bench]
    fn usual_position(b: &mut Bencher) {
        let case = "usual_position";

        let board = case.parse::<Board>().unwrap();
        let pos = Pos::from_str_unchecked("a1");

        b.iter(|| {
            let _ = board.set(pos);
        })
    }

    #[bench]
    fn deep_nested_position(b: &mut Bencher) {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . X . . . . . . . 12
        11 . . . . . . . . . . . . X . . 11
        10 . . . . . O . . . . . . O X . 10
         9 . . O X . . X O X X X O X . . 9
         8 . . . X O O . X O . . . . . . 8
         7 . O . . X X . . . X . . . . . 7
         6 . . . . . . X . . . X X O . . 6
         5 . . . . O . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let board = case.parse::<Board>().unwrap();
        let pos = Pos::from_str_unchecked("a1");

        b.iter(|| {
            let _ = board.set(pos);
        })
    }
}
