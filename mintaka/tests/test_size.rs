#[cfg(test)]
mod test_size {
    use mintaka::board::Board;
    use mintaka::pattern::{Pattern, Patterns};
    use mintaka::slice::{Slice, Slices};
    use mintaka::slice_pattern::{PatternPatch, SlicePatch};

    #[test]
    fn test_size() {
        assert_eq!(size_of::<Slice>(), 6);
        assert_eq!(size_of::<Slices>(), 432);
        assert_eq!(size_of::<Pattern>(), 8);
        assert_eq!(size_of::<Patterns>(), 1800);
        assert_eq!(size_of::<Board>(), 2248);
        assert_eq!(size_of::<PatternPatch>(), 2);
        assert_eq!(size_of::<SlicePatch>(), 30)
    }

}
