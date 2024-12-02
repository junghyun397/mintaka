#[cfg(test)]
mod test_slice {
    macro_rules! test_availability {
        ($case:expr, black=$black:expr, whthe=$white:expr) => {{
            let slice = Slice::from_str($case).unwrap();
            println!("black={}, white={}", slice.black_pattern_available, slice.white_pattern_available);
            assert_eq!(slice.black_pattern_available, $black);
            assert_eq!(slice.white_pattern_available, $white);
        }};
    }

    #[test]
    fn test_slice_validity() {
        // test_availability!(
        //     ". . . X O O O X O . . . . O",
        //     black=false,
        //     whthe=false
        // );
    }

}
