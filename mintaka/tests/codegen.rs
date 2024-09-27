use rand::Rng;

#[test]
mod codegen {
    use mintaka::notation::pos;
    use rand::Rng;

    #[test]
    fn generate_hash_table() {
        for _ in 0..pos::BOARD_SIZE {
            println!("0x{:016X},", rand::thread_rng().random::<u64>());
        }
    }

}
