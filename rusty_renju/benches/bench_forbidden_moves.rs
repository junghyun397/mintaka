#![feature(test)]

extern crate test;

mod bench_forbidden_moves {
    use indoc::indoc;
    use rusty_renju::board::Board;
    use rusty_renju::notation::pos::pos_unchecked;
    use test::Bencher;

    #[bench]
    fn clean_position(b: &mut Bencher) {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . . . . . . . . . . 10
         9 . . . X X . X O . . . . . . . 9
         8 . . . . . O O X . . . . . . . 8
         7 . . . . . . X O O . . . . . . 7
         6 . . . . . . . X O . . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let board = case.parse::<Board>().unwrap();
        let pos = pos_unchecked("i9");
        let validate_pos = pos_unchecked("i8");

        b.iter(|| {
            let board = board.set(pos);
            assert_eq!(board.patterns.field.white[validate_pos.idx_usize()].has_three(), false);
        })
    }

    #[bench]
    fn pseudo_position(b: &mut Bencher) {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . O . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . O . X . . . . . . . 10
         9 . . . . . X O X O . . . . . . 9
         8 . . . . . X . X . O . . . . . 8
         7 . . O X X X X O X . . . . . . 7
         6 . . . . . X . O . O . . . . . 6
         5 . . . . X O O . . . . . . . . 5
         4 . . . O . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let board = case.parse::<Board>().unwrap();
        let pos = pos_unchecked("h11");
        let validate_pos = pos_unchecked("g8");

        b.iter(|| {
            let board = board.set(pos);
            assert_eq!(board.patterns.field.black[validate_pos.idx_usize()].is_forbidden(), false);
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
         8 . . . . O O . X O . . . . . . 8
         7 . O . . X X . . . X . . . . . 7
         6 . . . . . . X . . . X X O . . 6
         5 . . . . O . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let board = case.parse::<Board>().unwrap();
        let pos = pos_unchecked("d8");
        let validate_pos = pos_unchecked("d7");

        b.iter(|| {
            let board = board.set(pos);
            assert_eq!(board.patterns.field.black[validate_pos.idx_usize()].is_forbidden(), false);
        })
    }
}
