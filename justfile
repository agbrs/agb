export CARGO_TARGET_DIR := env_var_or_default('CARGO_TARGET_DIR', justfile_directory() + "/target")

build: build-roms

test:
    just _all-crates _test-debug

clean:
    just _all-crates _clean

run-example example:
    just _build-example "{{example}}"
    mgba-qt "$CARGO_TARGET_DIR/thumbv4t-none-eabi/debug/examples/{{example}}"

run-example-release example:
    just _build-example-release "{{example}}"
    mgba-qt "$CARGO_TARGET_DIR/thumbv4t-none-eabi/release/examples/{{example}}"

run-game game:
    (cd "examples/{{game}}" && cargo run --release)

run-game-debug game:
    (cd "examples/{{game}}" && cargo run)

ci: && build-roms build-book
    just _all-crates _build
    just _all-crates _test-debug
    just _all-crates _test-release
    just _all-crates _clippy

build-roms:
    just _build-rom "examples/the-purple-night" "PURPLENIGHT"
    just _build-rom "examples/the-hat-chooses-the-wizard" "HATWIZARD"

    just _build-rom "book/games/pong" "PONG"

    (cd examples/target && zip examples.zip examples/*.gba)

build-book:
    (cd book && mdbook build)

update-lockfiles:
    bash .github/scripts/update-lockfiles.sh

_build-rom folder name:
    #!/usr/bin/env bash
    set -euxo pipefail

    GAME_FOLDER="{{folder}}"
    INTERNAL_NAME="{{name}}"

    GAME_NAME="$(basename "$GAME_FOLDER")"

    TARGET_FOLDER="${CARGO_TARGET_DIR:-$GAME_FOLDER/target}"
    GBA_FILE="$TARGET_FOLDER/$GAME_NAME.gba"

    (cd "$GAME_FOLDER" && cargo build --release --target thumbv4t-none-eabi)

    mkdir -p examples/target/examples

    arm-none-eabi-objcopy -O binary "$TARGET_FOLDER/thumbv4t-none-eabi/release/$GAME_NAME" "$GBA_FILE"
    gbafix -p "-t${INTERNAL_NAME:0:12}" "-c${INTERNAL_NAME:0:4}" -mGC "$GBA_FILE"

    cp -v "$GBA_FILE" "examples/target/examples/$GAME_NAME.gba"

_all-crates target:
    for CARGO_PROJECT_FILE in agb-*/Cargo.toml agb/Cargo.toml examples/*/Cargo.toml book/games/*/Cargo.toml; do \
        PROJECT_DIR=$(dirname "$CARGO_PROJECT_FILE"); \
        just "{{target}}" "$PROJECT_DIR" || exit $?; \
    done

_build crate:
    (cd "{{crate}}" && cargo build)
_test-release crate:
    {{ if crate =~ 'agb.*' { "cd " + crate + " && cargo test --release" } else { "" } }}
_test-debug crate:
    {{ if crate =~ 'agb.*' { "cd " + crate + " && cargo test" } else { "" } }}
_clippy crate:
    {{ if crate =~ 'agb.*' { "cd " + crate + " && cargo clippy" } else { "" } }}
_clean crate:
    (cd "{{crate}}" && cargo clean)

_build-example example:
    (cd agb && cargo build "--example={{example}}")
_build-example-release example:
    (cd agb && cargo build "--example={{example}}" --release)