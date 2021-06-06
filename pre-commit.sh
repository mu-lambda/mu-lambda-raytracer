#! /bin/sh
set -e
cargo test 
git diff --name-only --staged | grep .rs$ | xargs rustfmt -l | xargs git add 
