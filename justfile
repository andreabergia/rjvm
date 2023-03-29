# https://github.com/casey/just

default: build test lint

build:
    cargo build

test:
    RUST_LOG=trace cargo nextest run

test-verbose:
    RUST_LOG=trace cargo nextest run --no-capture

lint:
    cargo clippy --fix --allow-dirty --allow-staged

clean:
    cargo clean

fmt:
    cargo +nightly fmt

generate-test-classes:
    cd ./reader/tests/resources && ./compile.sh
    cd ./vm/tests/resources && ./compile.sh

count-lines:
    wc -l */{src,tests}/**/*.rs */{src,tests}/*.rs */tests/resources/**/*.java

miri:
    cargo clean
    cargo +nightly miri test
