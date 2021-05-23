#! /bin/sh
set -e
rustfmt src/*.rs -l | xargs git add 
cargo test 
