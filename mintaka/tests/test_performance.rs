#[cfg(test)]
mod test_performance {
    use mintaka::board::Board;
    use mintaka::formation::{Formation, Formations};
    use mintaka::slice::{Slice, Slices};

    #[test]
    fn test_size() {
        println!("slice size: {} bytes", size_of::<Slice>());
        println!("slices size: {} bytes", size_of::<Slices>());
        println!("formation size: {} bytes", size_of::<Formation>());
        println!("formations size: {} bytes", size_of::<Formations>());
        println!("board size: {} bytes", size_of::<Board>());
    }

}
