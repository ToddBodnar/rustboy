export RUST_BACKTRACE=1
cargo build --verbose && cargo test && cargo run "tests/blargg-gb/cpu_instrs/cpu_instrs.gb"
