#[cfg(test)]
mod test_slice {
    use mintaka::slice::*;
    use std::str::FromStr;

    macro_rules! test_validity {
        ($case:expr, black=$black:expr, whthe=$white:expr) => {{
            let black = Slice::from_str($case).unwrap().black_pattern_available;
            let white = Slice::from_str($case).unwrap().white_pattern_available;
            assert_eq!(black, $black);
            assert_eq!(white, $white);
        }};
    }

    #[test]
    fn test_slice_validity() {
        test_validity!(
            "O O X O . . O O . . X . . . X",
            black=true,
            whthe=true
        );
    }

}
