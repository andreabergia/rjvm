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
    cd ./reader/tests/resources && ./compile.sh

count-lines:
    wc -l */{src,tests}/**/*.rs */{src,tests}/*.rs */tests/resources/**/*.java
