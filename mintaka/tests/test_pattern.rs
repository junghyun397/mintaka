#[cfg(test)]
mod test_pattern {
    use mintaka::notation::color::Color;
    use mintaka::pattern::{CLOSED_FOUR_DOUBLE, CLOSED_FOUR_SINGLE, CLOSE_THREE, FIVE, INV_THREE_OVERLINE, OPEN_THREE};
    use mintaka::slice::Slice;
    use std::str::FromStr;

    fn test_patch(case: &str, expected: &str, color: Color, mask_kind: u8) {
        assert_eq!(case.len(), expected.len());

        let slice = Slice::from_str(case).unwrap();
        let patch = slice.calculate_slice_patch();

        let content_patch = patch.patch.iter()
            .take(slice.length as usize)
            .map(|pattern_patch| {
                let unit = match color {
                    Color::Black => pattern_patch.black_patch,
                    Color::White => pattern_patch.white_patch
                };

                if unit & mask_kind == mask_kind {
                    "V"
                } else {
                    "."
                }.to_string()
            })
            .reduce(|head, tail|
                format!("{head} {tail}")
            )
            .unwrap();

        assert_eq!(expected, content_patch);
    }

    fn test_patch_both_flow(case: &str, expected: &str, color: Color, mask_kind: u8) {
        test_patch(case, expected, color, mask_kind);
        test_patch(&case.chars().rev().collect::<String>(), &expected.chars().rev().collect::<String>(), color, mask_kind);
    }

    fn test_patch_both_color(case: &str, expected: &str, mask_kind: u8) {
        test_patch_both_flow(case, expected, Color::White, mask_kind);
        let replaced = case.replace("O", "@").replace("X", "O").replace("@", "X");
        test_patch_both_flow(&replaced, expected, Color::Black, mask_kind);
    }

    #[test]
    fn basic_three() {
        test_patch_both_color(
            ". . . O O . . .",
            ". V V . . V V .",
            OPEN_THREE
        );

        test_patch_both_color(
            ". . . O O . . .",
            ". . V . . V . .",
            CLOSE_THREE,
        );

        test_patch_both_color(
            ". . O . O . .",
            ". V . V . V .",
            OPEN_THREE
        );

        test_patch_both_color(
            ". . O . O . .",
            ". V . V . V .",
            CLOSE_THREE
        );
    }

    #[test]
    fn complex_three() {
        todo!()
    }

    #[test]
    fn basic_closed_four() {
        test_patch_both_color(
            ". . . O O O . . .",
            ". V . . . . . V .",
            CLOSED_FOUR_SINGLE
        );

        test_patch_both_color(
            "X O O O . . .",
            ". . . V V .",
            CLOSED_FOUR_SINGLE
        );

        test_patch_both_color(
            "X . O O O . . .",
            ". V . . . . V .",
            CLOSED_FOUR_SINGLE
        );

        todo!()
    }

    #[test]
    fn complex_four() {
        todo!()
    }

    #[test]
    fn basic_open_four() {
        todo!()
    }

    #[test]
    fn complex_open_four() {
        todo!()
    }

    #[test]
    fn double_four() {
        test_patch_both_color(
            ". O . O . O . O .",
            ". . . . V . . . .",
            CLOSED_FOUR_DOUBLE,
        );

        test_patch_both_color(
            ". . X O . O O . . O . .",
            ". . . . . . . V . . . .",
            CLOSED_FOUR_DOUBLE,
        );

        test_patch_both_color(
            ". . O O O . . . O O O . .",
            ". . . . . . V . . . . . .",
            CLOSED_FOUR_DOUBLE
        );
    }

    #[test]
    fn basic_five() {
        // TODO: basic test
        test_patch(
            ". . X X X X . .",
            ". . . . . . V .",
            Color::Black,
            FIVE
        );

        test_patch_both_color(
            ". . O O O O . .",
            ". V . . . . V .",
            FIVE
        );

        test_patch_both_color(
            ". . O O O . O . .",
            ". . . . . V . . .",
            FIVE
        );

        test_patch_both_color(
            ". . O O . O O . .",
            ". . . . V . . . .",
            FIVE
        );
    }

    #[test]
    fn complex_five() {
        test_patch_both_flow(
            ". X . X X X X . .",
            ". . . . . . . V .",
            Color::Black,
            FIVE
        );
    }

    #[test]
    fn overline() {
        test_patch_both_flow(
            ". . X X X . X X X . .",
            ". . . . . V . . . . .",
            Color::Black,
            INV_THREE_OVERLINE
        );

        test_patch_both_flow(
            ". . X X X . X X . .",
            ". . . . . V . . . .",
            Color::Black,
            INV_THREE_OVERLINE
        );

        test_patch_both_flow(
            ". . X X X X . X . .",
            ". . . . . . V . . .",
            Color::Black,
            INV_THREE_OVERLINE
        );
    }

}
