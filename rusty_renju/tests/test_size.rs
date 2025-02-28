#[cfg(test)]
mod test_size {
    use rusty_renju::board::Board;
    use rusty_renju::pattern::{Pattern, Patterns};
    use rusty_renju::slice::{Slice, Slices};
    use rusty_renju::slice_pattern::SlicePattern;

    #[test]
    fn test_size() {
        assert_eq!(size_of::<Slice>(), 16);
        assert_eq!(size_of::<Slices>(), 1152);
        assert_eq!(size_of::<Pattern>(), 4);
        assert_eq!(size_of::<Patterns>(), 1920);
        assert_eq!(size_of::<SlicePattern>(), 16);
        assert_eq!(size_of::<Board>(), 3136);
    }

}
