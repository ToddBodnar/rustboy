export RUST_BACKTRACE=1
cargo build --verbose && cargo test && cargo run "roms/blargg-gb/cpu_instrs.gb"
