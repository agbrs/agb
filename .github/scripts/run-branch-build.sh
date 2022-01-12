#!/usr/bin/env bash

set -e # Fail if any command fails

CARGO_PROJECT_FILES=( agb-*/Cargo.toml agb/Cargo.toml examples/*/Cargo.toml book/games/*/Cargo.toml )

for CARGO_PROJECT_FILE in "${CARGO_PROJECT_FILES[@]}"; do
    PROJECT_DIR=$(dirname "$CARGO_PROJECT_FILE")

    echo "Checking project $PROJECT_DIR"
    (cd "$PROJECT_DIR" && cargo build)

    if echo "$PROJECT_DIR" | grep -qE '^agb'; then
        echo "Running clippy on $PROJECT_DIR"
        (cd "$PROJECT_DIR" && cargo clippy)

        echo "Testing $PROJECT_DIR in debug mode"
        (cd "$PROJECT_DIR" && cargo test)

        echo "Testing $PROJECT_DIR and release mode"
        (cd "$PROJECT_DIR" && cargo test --release)
    fi
done
