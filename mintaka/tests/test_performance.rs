#[cfg(test)]
mod test_performance {
    use mintaka::board::Board;
    use mintaka::formation::{Formation, Formations};
    use mintaka::pattern::{FormationPatch, SlicePatch};
    use mintaka::slice::{Slice, Slices};

    #[test]
    fn test_size() {
        assert_eq!(size_of::<Slice>(), 6);
        assert_eq!(size_of::<Slices>(), 432);
        assert_eq!(size_of::<Formation>(), 8);
        assert_eq!(size_of::<Formations>(), 1800);
        assert_eq!(size_of::<Board>(), 2248);
        assert_eq!(size_of::<FormationPatch>(), 2);
        assert_eq!(size_of::<SlicePatch>(), 30)
    }

}
