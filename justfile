# https://github.com/casey/just

default: build test lint

build:
    cargo build

test:
    RUST_LOG=trace cargo test -- --nocapture

lint:
    cargo clippy --fix --allow-dirty --allow-staged

clean:
    cargo clean

generate-test-classes:
    cd ./tests/resources && ./compile.sh
