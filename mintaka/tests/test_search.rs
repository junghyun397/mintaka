#[cfg(test)]
mod test_search {
    macro_rules! search {
        ($board:expr) => {{
            let mut board = $board;
            let config = Config::default();

            let launched = AtomicBool::new(false);
            let aborted = AtomicBool::new(false);

            let mut game_agent = GameAgent::new(config, aborted.clone());
        }};
    }

    #[test]
    fn test_search() {
    }

}
