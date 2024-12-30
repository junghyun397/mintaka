#[cfg(test)]
mod codegen {
    use rand::Rng;
    use rusty_renju::notation::pos;

    #[test]
    fn generate_hash_table() {
        for _ in 0..pos::BOARD_SIZE {
            println!("0x{:016X},", rand::thread_rng().random::<u64>());
        }
    }

}
