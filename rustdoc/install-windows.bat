@echo off
cargo update
cargo install --path . --bin narc --force
cargo clippy
