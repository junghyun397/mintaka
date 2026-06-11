bin=mintaka_text_protocol_renju

cargo build --release -p mintaka_interface --features="text-protocol" --bin $bin

cp target/release/$bin bins/${bin}_$(git rev-parse HEAD)
mv target/release/$bin target/release/${bin}_target
