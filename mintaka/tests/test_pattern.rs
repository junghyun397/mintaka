#[cfg(test)]
mod test_pattern {
    use mintaka::notation::color::Color;
    use mintaka::pattern::{CLOSED_FOUR_DOUBLE, CLOSED_FOUR_SINGLE, CLOSE_THREE, FIVE, INV_THREE_OVERLINE, OPEN_FOUR, OPEN_THREE};
    use mintaka::slice::Slice;
    use std::str::FromStr;

    fn test(case: &str, expected: &str, color: Color, mask_kind: u8) {
        assert_eq!(case.len(), expected.len());

        let slice = Slice::from_str(case).unwrap();
        let patch = slice.calculate_slice_patch();

        let content_patch = patch.black_patch.iter()
            .zip(patch.white_patch.iter())
            .take(slice.length as usize)
            .enumerate()
            .map(|(idx, (black, white))| {
                let unit = match color {
                    Color::Black => black,
                    Color::White => white
                };

                if unit & mask_kind == mask_kind {
                    'V'
                } else if case.as_bytes()[idx * 2] != '.' as u8 {
                    case.as_bytes()[idx * 2] as char
                } else {
                    '.'
                }.to_string()
            })
            .reduce(|head, tail|
                format!("{head} {tail}")
            )
            .unwrap();

        assert_eq!(expected, content_patch);
    }

    fn test_both_flow(case: &str, expected: &str, color: Color, mask_kind: u8) {
        test(case, expected, color, mask_kind);
        test(&case.chars().rev().collect::<String>(), &expected.chars().rev().collect::<String>(), color, mask_kind);
    }

    fn test_both_color_both_flow(case: &str, expected: &str, mask_kind: u8) {
        test_both_flow(case, expected, Color::White, mask_kind);
        let case = &case.replace("O", "@").replace("X", "O").replace("@", "X");
        let expected = &expected.replace("O", "@").replace("X", "O").replace("@", "X");
        test_both_flow(case, expected, Color::Black, mask_kind);
    }

    #[test]
    fn basic_open_three() {
        test_both_color_both_flow(
            ". . . O O . . .",
            ". V V O O V V .",
            OPEN_THREE
        );

        test_both_color_both_flow(
            ". . O . O . .",
            ". V O V O V .",
            OPEN_THREE
        );
    }

    #[test]
    fn complex_open_three() {
        test_both_flow(
            "X . . X X . . .",
            "X . . X X V V .",
            Color::Black,
            OPEN_THREE
        );

        test_both_flow(
            "X . . . X X . . .",
            "X . . V X X V V .",
            Color::Black,
            OPEN_THREE
        );
    }

    #[test]
    fn basic_close_three() {
        test_both_flow(
            "X . . X X X . . .",
            "X . V X X X V V .",
            Color::Black,
            CLOSE_THREE
        );

        test_both_flow(
            "X . . X X X . . .",
            "X . V X X X V V .",
            Color::Black,
            CLOSE_THREE
        );
    }
    
    #[test]
    fn complex_close_three() { 
    }

    #[test]
    fn basic_closed_four() {
        test_both_color_both_flow(
            ". . O O O . .",
            "V . O O O . V",
            CLOSED_FOUR_SINGLE
        );

        test_both_color_both_flow(
            "O O O . .",
            "O O O V V",
            CLOSED_FOUR_SINGLE
        );

        test_both_color_both_flow(
            "X . O O O . .",
            "X V O O O . V",
            CLOSED_FOUR_SINGLE
        );
    }

    #[test]
    fn complex_closed_four() {
    }

    #[test]
    fn basic_open_four() {
        test_both_color_both_flow(
            ". . O O O . .",
            ". V O O O V .",
            OPEN_FOUR
        );

        test_both_color_both_flow(
            ". O O . O .",
            ". O O V O .",
            OPEN_FOUR
        )
    }

    #[test]
    fn complex_open_four() {
        test_both_flow(
            "X . . X X X . .",
            "X . . X X X V .",
            Color::Black,
            OPEN_FOUR
        );
    }

    #[test]
    fn double_four() {
        test_both_color_both_flow(
            ". O . O . O . O .",
            ". O . O V O . O .",
            CLOSED_FOUR_DOUBLE,
        );

        test_both_color_both_flow(
            ". . O O O . . . O O O . .",
            ". . O O O . V . O O O . .",
            CLOSED_FOUR_DOUBLE
        );

        test_both_color_both_flow(
            "O . O O . . O",
            "O . O O V . O",
            CLOSED_FOUR_DOUBLE,
        );
    }

    #[test]
    fn basic_five() {
        test_both_color_both_flow(
            ". O O O O .",
            "V O O O O V",
            FIVE
        );

        test_both_color_both_flow(
            "O O O O .",
            "O O O O V",
            FIVE
        );

        test_both_color_both_flow(
            "O O O . O",
            "O O O V O",
            FIVE
        );

        test_both_color_both_flow(
            "O O . O O",
            "O O V O O",
            FIVE
        );
    }

    #[test]
    fn complex_five() {
        test_both_flow(
            "X . X X X X .",
            "X . X X X X V",
            Color::Black,
            FIVE
        );

        test_both_flow(
            "X X X . X X",
            "X X X . X X",
            Color::Black,
            FIVE
        );

        test_both_flow(
            "O . O O O O .",
            "O V O O O O V",
            Color::White,
            FIVE
        );
    }

    #[test]
    fn overline() {
        test(
            "X X X . X X",
            "X X X V X X",
            Color::Black,
            INV_THREE_OVERLINE
        );

        test_both_flow(
            "X X X X . X",
            "X X X X V X",
            Color::Black,
            INV_THREE_OVERLINE
        );
    }

}
