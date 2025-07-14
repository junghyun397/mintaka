#[cfg(test)]
mod playground {
    use mintaka::protocol::response::Response::Begins;
    use rusty_renju::board::Board;
    use rusty_renju::memo::hash_key::HashKey;
    use rusty_renju::utils::byte_size::ByteSize;
    use rusty_renju::{board, history};

    #[test]
    fn test_serde() {
        let board = board!("h8", "h7", "h6", "a8", "h9");
        let board_str = serde_json::to_string(&board).unwrap();

        println!("{}", board_str);

        let board: Board = serde_json::from_str(&board_str).unwrap();

        println!("{}", board);
    }

    #[test]
    fn test_serde_2() {
        let response = Begins {
            workers: 0,
            running_time: Default::default(),
            tt_size: ByteSize::from_mib(100),
        };

        let str = serde_json::to_string(&response).unwrap();

        println!("{}", str);
    }

    #[test]
    fn test_serde_3() {
        let hash_key = HashKey::default();
        let mid = serde_json::to_string(&hash_key).unwrap();
        println!("{}", serde_json::from_str::<HashKey>(&mid).unwrap());
    }

    #[test]
    fn test_serde_4() {
        let history = history!("a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9");
        let json = serde_json::to_string(&history).unwrap();
        println!("{}", json);
    }

}