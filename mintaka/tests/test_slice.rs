#[cfg(test)]
mod test_slice {
    use mintaka::slice::*;
    use std::str::FromStr;

    macro_rules! test_availability {
        ($case:expr, black=$black:expr, whthe=$white:expr) => {{
            let black = Slice::from_str($case).unwrap().black_pattern_available;
            let white = Slice::from_str($case).unwrap().white_pattern_available;
            assert_eq!(black, $black);
            assert_eq!(white, $white);
        }};
    }

    #[test]
    fn test_slice_validity() {
        test_availability!(
            ". . . . . X O O O X O . . . O",
            black=false,
            whthe=false
        );
    }

}
