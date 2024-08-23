#[cfg(test)]
mod test_performance {
    use resrenju::board::Board;
    use resrenju::formation::{FormationPair, FormationPairs};
    use resrenju::slice::{Slice, Slices};

    #[test]
    fn test_size() {
        println!("slice size: {} bytes", size_of::<Slice>());
        println!("slices size: {} bytes", size_of::<Slices>());
        println!("formation pair size: {} bytes", size_of::<FormationPair>());
        println!("formations size: {} bytes", size_of::<FormationPairs>());
        println!("board size: {} bytes", size_of::<Board>());
    }

}
