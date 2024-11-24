#[cfg(test)]
mod test_slice_pattern {
    use mintaka::notation::color::*;
    use mintaka::pattern::*;
    use mintaka::slice::*;
    use std::str::FromStr;

    macro_rules! test {
        ($case:expr, $expected:expr, $color:expr, $mask:expr, $result:expr) => {{
            assert_eq!($case.len(), $expected.len());

            let slice = Slice::from_str($case).unwrap();
            let slice_pattern = slice.calculate_slice_pattern();

            let content_pattern = slice_pattern.black_patterns.iter()
                .zip(slice_pattern.white_patterns.iter())
                .take(slice.length as usize)
                .enumerate()
                .map(|(idx, (black, white))| {
                    let unit = match $color {
                        Color::Black => black,
                        Color::White => white,
                    };

                    if unit & $mask == $result {
                        'V'
                    } else if $case.as_bytes()[idx * 2] != b'.' {
                        $case.as_bytes()[idx * 2] as char
                    } else {
                        '.'
                    }
                    .to_string()
                })
                .reduce(|head, tail|
                    format!("{head} {tail}")
                )
                .unwrap();

            assert_eq!(&content_pattern, $expected);
        }};
    }

    macro_rules! test_both_flow {
        ($case:expr, $expected:expr, $color:expr, $mask:expr, $result:expr) => {{
            test!($case, $expected, $color, $mask, $result);
            test!(&$case.chars().rev().collect::<String>(), &$expected.chars().rev().collect::<String>(), $color, $mask, $result);
        }};
    }

    fn invert_color(case: &str) -> String {
        case.replace("O", "@").replace("X", "O").replace("@", "X")
    }

    macro_rules! test_pattern {
        (
            color = both,
            case = $case:expr,
            $(open_three = $open_three:expr,)?
            $(closed_four_single = $closed_four_single:expr,)?
            $(closed_four_double = $closed_four_double:expr,)?
            $(open_four = $open_four:expr,)?
            $(close_three = $close_three:expr,)?
            $(five = $five:expr,)?
            $(overline = $overline:expr,)?
        ) => {
            test_pattern!(
                color = Color::White,
                case = $case,
                $(open_three = $open_three,)?
                $(closed_four_single = $closed_four_single,)?
                $(closed_four_double = $closed_four_double,)?
                $(open_four = $open_four,)?
                $(close_three = $close_three,)?
                $(five = $five,)?
                $(overline = $overline,)?
            );

            test_pattern!(
                color = Color::Black,
                case = &invert_color($case),
                $(open_three = &invert_color($open_three),)?
                $(closed_four_single = &invert_color($closed_four_single),)?
                $(closed_four_double = &invert_color($closed_four_double),)?
                $(open_four = &invert_color($open_four),)?
                $(close_three = &invert_color($close_three),)?
                $(five = &invert_color($five),)?
                $(overline = &invert_color($overline),)?
            );
        };
        (
            color = $color:expr,
            case = $case:expr,
            $(open_three = $open_three:expr,)?
            $(closed_four_single = $closed_four_single:expr,)?
            $(closed_four_double = $closed_four_double:expr,)?
            $(open_four = $open_four:expr,)?
            $(close_three = $close_three:expr,)?
            $(five = $five:expr,)?
            $(overline = $overline:expr,)?
        ) => {
            $(test_both_flow!($case, $open_three, $color, OPEN_THREE, OPEN_THREE);)?

            $(test_both_flow!($case, $closed_four_single, $color, CLOSED_FOUR_DOUBLE, CLOSED_FOUR_SINGLE);)?

            $(test_both_flow!($case, $closed_four_double, $color, CLOSED_FOUR_DOUBLE, CLOSED_FOUR_DOUBLE);)?

            $(test_both_flow!($case, $open_four, $color, OPEN_FOUR, OPEN_FOUR);)?

            $(test_both_flow!($case, $close_three, $color, CLOSE_THREE, CLOSE_THREE);)?

            $(test_both_flow!($case, $five, $color, FIVE, FIVE);)?

            $(test_both_flow!($case, $overline, $color, OVERLINE, OVERLINE);)?
        };
    }

    #[test]
    fn three() {
        test_pattern!(
            color = both,
            case                = ". . . O O . . .",
            open_three          = ". V V O O V V .",
        );

        test_pattern!(
            color = both,
            case                = "X . O O . . .",
            open_three          = "X . O O V V .",
        );

        test_pattern!(
            color = both,
            case                = "X . . O O . . .",
            open_three          = "X . V O O V V .",
        );

        test_pattern!(
            color = both,
            case                = ". . O . O . .",
            open_three          = ". V O V O V .",
        );

        test_pattern!(
            color = both,
            case                = "X . O . O . .",
            open_three          = "X . O V O V .",
        );
        
        test_pattern!(
            color = both,
            case                = ". O . . O .",
            open_three          = ". O V V O .",
        );
    }

    #[test]
    fn complex_three() {
        test_pattern!(
            color = Color::Black,
            case                = "X . . X X . . .",
            open_three          = "X . . X X V V .",
            closed_four_single  = "X V V X X . . .",
        );

        test_pattern!(
            color = Color::Black,
            case                = "X . . . X X . . .",
            open_three          = "X . . V X X V V .",
        );

        test_pattern!(
            color = Color::Black,
            case                = "X . . . X X . . X",
            open_three          = "X . . . X X . . X",
        );

        test_pattern!(
            color = Color::Black,
            case                = "X . . . X X . . . X",
            open_three          = "X . . V X X V . . X",
        );

        test_pattern!(
            color = Color::Black,
            case                = "X . . X . X . .",
            open_three          = "X . . X V X V .",
        );

        test_pattern!(
            color = Color::Black,
            case                = "X . . X . X . . X",
            open_three          = "X . . X . X . . X",
        );
    }

    #[test]
    fn four() {
        test_pattern!(
            color = both,
            case                = "O O O . .",
            closed_four_single  = "O O O V V",
        );

        test_pattern!(
            color = both,
            case                = ". O O O .",
            closed_four_single  = "V O O O V",
        );

        test_pattern!(
            color = both,
            case                = ". . O O O . .",
            closed_four_single  = "V . O O O . V",
            open_four           = ". V O O O V .",
            close_three         = ". V O O O V .",
        );

        test_pattern!(
            color = both,
            case                = "X . O O O . .",
            closed_four_single  = "X V O O O . V",
            open_four           = "X . O O O V .",
            close_three         = "X V O O O V V",
        );

        test_pattern!(
            color = both,
            case                = ". . O O O . .",
            closed_four_single  = "V . O O O . V",
            open_four           = ". V O O O V .",
            close_three         = ". V O O O V .",
        );

        test_pattern!(
            color = both,
            case                = ". O O . . O .",
            closed_four_single  = ". O O V V O .",
        );

        test_pattern!(
            color = both,
            case                = ". O O . O .",
            closed_four_single  = "V O O . O V",
            open_four           = ". O O V O .",
            close_three         = "V O O V O V",
        );
    }

    #[test]
    fn complex_four_black() {
        test_pattern!(
            color = Color::Black,
            case                = "X . . X X X . . .",
            closed_four_single  = "X . V X X X . V .",
            open_four           = "X . . X X X V . .",
            close_three         = "X . V X X X V V .",
        );

        test_pattern!(
            color = Color::Black,
            case                = ". X . X X . X .",
            closed_four_single  = "V X V X X V X V",
            closed_four_double  = ". X . X X . X .",
            open_four           = ". X . X X . X .",
            close_three         = ". X . X X . X .",
        );

        test_pattern!(
            color = Color::Black,
            case                = ". X . X . X X . .",
            closed_four_single  = ". X . X V X X V .",
            closed_four_double  = ". X . X . X X . .",
            open_four           = ". X . X . X X . .",
            close_three         = ". X . X . X X . .",
        );

        test_pattern!(
            color = Color::Black,
            case                = ". X X . . X X .",
            closed_four_single  = ". X X . . X X .",
        );
    }

    #[test]
    fn complex_four_white() {
        test_pattern!(
            color = Color::White,
            case                = "O . . O O O",
            closed_four_single  = "O V V O O O",
            closed_four_double  = "O . . O O O",
        );

        test_pattern!(
            color = Color::White,
            case                = ". O O O . . O",
            closed_four_single  = "V O O O V V O",
            closed_four_double  = ". O O O . . O",
            open_four           = ". O O O V . O",
            close_three         = "V O O O V V O",
        );

        test_pattern!(
            color = Color::White,
            case                = "O O O . . O O O",
            closed_four_single  = "O O O V V O O O",
            closed_four_double  = "O O O . . O O O",
        );

        test_pattern!(
            color = Color::White,
            case                = ". . O . O O . O",
            closed_four_single  = ". V O . O O V O",
            closed_four_double  = ". . O . O O . O",
            open_four           = ". . O V O O . O",
        );
    }

    #[test]
    fn double_four() {
        test_pattern!(
            color = both,
            case                = ". O . O . O . O .",
            closed_four_single  = ". O V O . O V O .",
            closed_four_double  = ". O . O V O . O .",
        );

        test_pattern!(
            color = both,
            case                = ". O . O . O . O . O . O .",
            closed_four_single  = ". O V O . O . O . O V O .",
            closed_four_double  = ". O . O V O V O V O . O .",
        );

        test_pattern!(
            color = both,
            case                = ". O . O O . . O .",
            closed_four_single  = "V O . O O . V O .",
            closed_four_double  = ". O . O O V . O .",
            open_four           = ". O V O O . . O .",
        );

        test_pattern!(
            color = both,
            case                = ". O O . O . . O O .",
            closed_four_single  = "V O O . O . V O O .",
            closed_four_double  = ". O O . O V . O O .",
            open_four           = ". O O V O . . O O .",
        );

        test_pattern!(
            color = Color::White,
            case                = "O . O O . . O . O O",
            closed_four_single  = "O V O O . . O V O O",
            closed_four_double  = "O . O O V V O . O O",
        );

        test_pattern!(
            color = both,
            case                = ". . O O O . . . O O O . .",
            closed_four_single  = "V . O O O . . . O O O . V",
            closed_four_double  = ". . O O O . V . O O O . .",
            open_four           = ". V O O O V . V O O O V .",
        );

        test_pattern!(
            color = Color::White,
            case                = "O . O . O O . . O O . O . . .",
            closed_four_single  = "O V O . O O . . O O . O V . .",
            closed_four_double  = "O . O . O O V V O O . O . . .",
            open_four           = "O . O V O O . . O O V O . . .",
        );

        test_pattern!(
            color = Color::Black,
            case                = "X . X . X X . . X X . X . . .",
            closed_four_single  = "X . X V X X V V X X . X V . .",
            closed_four_double  = "X . X . X X . . X X . X . . .",
            open_four           = "X . X . X X . . X X V X . . .",
        );
    }

    #[test]
    fn five() {
        test_pattern!(
            color = both,
            case                = ". O O O O .",
            five                = "V O O O O V",
        );

        test_pattern!(
            color = both,
            case                = "O O O . O",
            five                = "O O O V O",
        );

        test_pattern!(
            color = both,
            case                = "O O . O O",
            five                = "O O V O O",
        );
    }

    #[test]
    fn overline() {
        test_pattern!(
            color = Color::Black,
            case                = "X . X X X X .",
            five                = "X . X X X X V",
            overline            = "X V X X X X .",
        );

        test_pattern!(
            color = Color::Black,
            case                = "X X X . X X",
            five                = "X X X . X X",
            overline            = "X X X V X X",
        );

        test_pattern!(
            color = Color::Black,
            case                = "X X X X . X",
            five                = "X X X X . X",
            overline            = "X X X X V X",
        );
    }

}
