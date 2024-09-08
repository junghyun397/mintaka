#[cfg(test)]
mod test_forbid_complex {

    #[test]
    fn pseudo_double_three() {
        todo!()
    }

    #[test]
    fn double_three() {
        todo!()
    }

    #[test]
    fn pseudo_double_four() {
        todo!()
    }

    #[test]
    fn double_four() {
        todo!()
    }

    #[test]
    fn recursive_double_three() {
        let origin = "\
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . O O . . . . . . . 10
         9 . . . . . . X X . . . . . . . 9
         8 . . . . O X X X X O . . . . . 8
         7 . . . . O X X X X O . . . . . 7
         6 . . . . . . X X . . . . . . . 6
         5 . . . . . . O O . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O";
        let excepted = "\
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . O O . . . . . . . 10
         9 . . . . . 3 X X 3 . . . . . . 9
         8 . . . . O X X X X O . . . . . 8
         7 . . . . O X X X X O . . . . . 7
         6 . . . . . 3 X X 3 . . . . . . 6
         5 . . . . . . O O . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O";
    }

}
