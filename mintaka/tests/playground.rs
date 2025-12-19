#[cfg(test)]
mod playground {
    use indoc::indoc;
    use rusty_renju::board::Board;
    use rusty_renju::slice_pattern_count::PADDED_SLICE_AMOUNT;

    #[test]
    fn avs() {
        let f = 6000u64 as f64;
        println!("{}", f / 6000.0 * 100.0);
    }

    #[test]
    fn nn() {
    }

    #[test]
    fn mg() {
        println!("{}", PADDED_SLICE_AMOUNT);
    }

    #[test]
    fn playground() {
        let case = indoc! {"
   A B C D E F G H I J K L M N O
15 . . . . . . . . . . . . . . . 15
14 . . . . . . . . . . . . . . . 14
13 . . . . . . . . . . . . . . . 13
12 . . . . . . . . . . . . . . . 12
11 . . . . . . . . . . . . . . . 11
10 . . . . . X . O . . . . . . . 10
 9 . . . . O O O X O . . . . . . 9
 8 . . . . . X . X O O . . . . . 8
 7 . . . . O X X X X O . . . . . 7
 6 . . . . . X . X O X . . . . . 6
 5 . . . . O O X O O X O . . . . 5
 4 . . . X . X 4 X . X . X . . . 4
 3 . . . . O . O . O O . . . . . 3
 2 . . . . . . . . . X . . . . . 2
 1 . . . . . . . . . . . . . . . 1
   A B C D E F G H I J K L M N O
        "};

        let mut board = case.parse::<Board>().unwrap();

        println!("{}", board.to_string_with_pattern_analysis());
    }

}
