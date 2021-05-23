#! /bin/sh
set -e
cargo test 
rustfmt src/*.rs -l | xargs git add 
