#[cfg(test)]
mod test_slice {
    use mintaka::slice::*;
    use std::str::FromStr;

    macro_rules! test_validity {
        ($case:expr, $expected:expr) => {{
            let validity = Slice::from_str($case).unwrap().is_valid_pattern();
            assert_eq!(validity, $expected);
        }};
    }

    #[test]
    fn test_playground() {
        let w = 0b0100_0110u16;
        let b = 0b0010_1001u16;

        // (a & ~(b << 1) & ~(b >> 1)) | (b & ~(a << 1) & ~(a >> 1)) != 0
    }

    #[test]
    fn test_slice_validity() {
        test_validity!(
            "O . X . . . . . . O X",
            true
        );
    }

}
