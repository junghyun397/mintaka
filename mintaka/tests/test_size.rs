#[cfg(test)]
mod test_size {
    use mintaka::board::Board;
    use mintaka::pattern::{Pattern, Patterns};
    use mintaka::slice::{Slice, Slices};
    use mintaka::slice_pattern::SlicePattern;

    #[test]
    fn test_size() {
        assert_eq!(size_of::<Slice>(), 16);
        assert_eq!(size_of::<Slices>(), 1152);
        assert_eq!(size_of::<Pattern>(), 8);
        assert_eq!(size_of::<Patterns>(), 1840);
        assert_eq!(size_of::<SlicePattern>(), 34);
        assert_eq!(size_of::<Board>(), 3040);
    }

}
