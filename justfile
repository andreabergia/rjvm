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
    cd ./reader/tests/resources && rm -f *.class && ./compile.sh
    cd ./vm/tests/resources && rm -f *.class && ./compile.sh

count-lines:
    wc -l */{src,tests}/**/*.rs */{src,tests}/*.rs */tests/resources/**/*.java

miri:
    cargo clean
    MIRIFLAGS="-Zmiri-disable-isolation -Zmiri-report-progress" cargo +nightly miri test

prof-vm-integration: clean
    cd vm && CARGO_PROFILE_BENCH_DEBUG=true cargo flamegraph --test integration --root && open flamegraph.svg

find-unused-dependencies:
    cargo +nightly udeps --all-targets
