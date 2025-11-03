#!/bin/bash

# AVX512
RUSTFLAGS="-C target-feature=+avx2,+bmi2,+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vl,+avx512vnni" \
  cargo build -p mintaka_interface --release --bin pbrain_mintaka

mv target/release/pbrain_mintaka target/release/pbrain_mintaka_avx512vnni_bmi2

# AVX2
RUSTFLAGS="-C target-feature=+avx2,+bmi2" \
  cargo build -p mintaka_interface --release --bin pbrain_mintaka

mv target/release/pbrain_mintaka target/release/pbrain_mintaka_avx2

# baseline
cargo build -p mintaka_interface --release --bin pbrain_mintaka
