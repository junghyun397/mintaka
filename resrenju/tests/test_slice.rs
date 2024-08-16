#[cfg(test)]
mod test_slice {
    use resrenju::notation::pos::Pos;
    use resrenju::slice::{Direction, Slices};

    #[test]
    fn basic_test() {
        let slices = Slices::empty();
        println!("pos: {}", slices.access_slice(
            "m9".parse::<Pos>().unwrap(),
            Direction::Descending
        ).unwrap().start_pos);
    }

    #[test]
    fn basic_three() {
        todo!()
    }

    #[test]
    fn complex_three() {
        todo!()
    }

    #[test]
    fn basic_four() {
        todo!()
    }

    #[test]
    fn complex_four() {
        todo!()
    }

    #[test]
    fn basic_open_four() {
        todo!()
    }

    #[test]
    fn complex_open_four() {
        todo!()
    }

    #[test]
    fn basic_five() {
        todo!()
    }

    #[test]
    fn double_four_forbid() {
        todo!()
    }

    #[test]
    fn overline_forbid() {
        todo!()
    }

}
