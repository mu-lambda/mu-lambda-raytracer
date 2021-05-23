#! /bin/sh
rustfmt src/*.rs || exit 1
cargo test || exit 1
