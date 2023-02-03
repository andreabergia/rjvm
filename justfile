# https://github.com/casey/just

default: build test lint

build:
    cargo build

test:
    cargo test

lint:
    cargo clippy --fix --allow-dirty --allow-staged

generate-test-classes:
    cd ./tests/resources && ./compile.sh
