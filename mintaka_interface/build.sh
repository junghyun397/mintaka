#!/bin/bash

cargo build --release --bin pbrain_mintaka_baseline

RUSTFLAGS="-C target-feature=+avx2" \
  cargo build --release --bin pbrain_mintaka_avx2

RUSTFLAGS="-C target-feature=+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vl,+avx512vnni" \
  cargo build --release --bin pbrain_mintaka_avx512_vnni
