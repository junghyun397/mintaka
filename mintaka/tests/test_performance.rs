#[cfg(test)]
mod test_performance {
    use mintaka::board::Board;
    use mintaka::formation::{Cell, Cells};
    use mintaka::slice::{Slice, Slices};

    #[test]
    fn test_size() {
        println!("slice size: {} bytes", size_of::<Slice>());
        println!("slices size: {} bytes", size_of::<Slices>());
        println!("formation pair size: {} bytes", size_of::<Cell>());
        println!("formations size: {} bytes", size_of::<Cells>());
        println!("board size: {} bytes", size_of::<Board>());
    }

}
