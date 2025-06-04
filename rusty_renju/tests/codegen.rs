#[cfg(test)]
mod codegen {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use rusty_renju::notation::pos;

    fn generate_hash_table() {
        let seed = 42;
        let mut rng = StdRng::seed_from_u64(seed);

        for _ in 0..pos::BOARD_SIZE {
            println!("0x{:016X},", rng.random::<u64>());
        }

        println!("---");

        for _ in 0..pos::BOARD_SIZE {
            println!("0x{:016X},", rng.random::<u64>());
        }

        println!("---");

        println!("0x{:016X},", rng.random::<u64>());
    }

}
