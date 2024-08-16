#[cfg(test)]
mod test_board {
    use resrenju::board::Board;
    use resrenju::slice::{Slice, Slices};

    #[test]
    fn test_size() {
        println!("board size: {} bits", size_of::<Board>() * 8);
        println!("slice size: {} bits", size_of::<Slice>() * 8);
        println!("slices size: {} bits", size_of::<Slices>() * 8);
    }

    #[test]
    fn test_set() {
        todo!()
    }

    #[test]
    fn test_set_mut() {
        todo!()
    }

}
