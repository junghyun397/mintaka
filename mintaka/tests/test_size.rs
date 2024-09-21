#[cfg(test)]
mod test_size {
    use mintaka::board::Board;
    use mintaka::pattern::{Pattern, Patterns};
    use mintaka::slice::{Slice, Slices};
    use mintaka::slice_pattern::SlicePattern;

    #[test]
    fn test_size() {
        assert_eq!(size_of::<Slice>(), 6);
        assert_eq!(size_of::<Slices>(), 432);
        assert_eq!(size_of::<Pattern>(), 8);
        assert_eq!(size_of::<Patterns>(), 1803);
        assert_eq!(size_of::<SlicePattern>(), 32);
        assert_eq!(size_of::<Board>(), 2256);
    }

}