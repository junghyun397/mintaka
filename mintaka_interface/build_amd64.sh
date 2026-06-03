#!/bin/bash

# AVX512vnni
RUSTFLAGS="-C target-cpu=x86-64-v4 -C target-feature=+avx512vnni" \
  cargo build --release -p mintaka_interface --bin pbrain-mintaka_renju-15

mv target/release/pbrain-mintaka_renju-15 target/release/pbrain-mintaka_renju-15_avx512vnni

# AVX2+BMI2
RUSTFLAGS="-C target-cpu=x86-64-v3" \
  cargo build --release -p mintaka_interface --bin pbrain-mintaka_renju-15

mv target/release/pbrain_mintaka target/release/pbrain_mintaka_avx2
