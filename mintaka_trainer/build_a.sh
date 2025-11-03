#!/bin/bash

cargo build -p mintaka_interface --bin mintaka_text_protocol --release
mv target/release/mintaka_text_protocol target/release/mintaka_text_protocol_a
