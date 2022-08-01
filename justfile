export CARGO_TARGET_DIR := env_var_or_default('CARGO_TARGET_DIR', justfile_directory() + "/target")
CLIPPY_ARGUMENTS := "-Dwarnings -Dclippy::all -Aclippy::empty-loop"

build: build-roms

build-debug:
    just _build-debug agb
build-release:
    just _build-release agb
clippy:
    just _all-crates _clippy

test:
    just _test-debug agb
    just _test-debug agb-fixnum
    just _test-debug-arm agb

test-release:
    just _test-release agb
    just _test-release-arm agb

doctest-agb:
    (cd agb && cargo test --doc -Z doctest-xcompile)

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

ci: build-debug clippy test build-release test-release doctest-agb build-roms build-book

build-roms:
    just _build-rom "examples/the-purple-night" "PURPLENIGHT"
    just _build-rom "examples/the-hat-chooses-the-wizard" "HATWIZARD"
    just _build-rom "examples/hyperspace-roll" "HYPERSPACE"

    just _build-rom "book/games/pong" "PONG"

    (cd examples/target && zip examples.zip examples/*.gba)

build-book:
    (cd book && mdbook build)

update-lockfiles:
    bash .github/scripts/update-lockfiles.sh

update-linker-scripts:
    find -type f -name gba.ld | grep -v ./agb/gba.ld | xargs -n1 cp -v -- agb/gba.ld
    find -type f -name gba_mb.ld | grep -v ./agb/gba_mb.ld | xargs -n1 cp -v -- agb/gba_mb.ld

_build-rom folder name:
    #!/usr/bin/env bash
    set -euxo pipefail

    GAME_FOLDER="{{folder}}"
    INTERNAL_NAME="{{name}}"

    GAME_NAME="$(basename "$GAME_FOLDER")"

    TARGET_FOLDER="${CARGO_TARGET_DIR:-$GAME_FOLDER/target}"
    GBA_FILE="$TARGET_FOLDER/$GAME_NAME.gba"

    (cd "$GAME_FOLDER" && cargo build --release --target thumbv4t-none-eabi && cargo clippy --release --target thumbv4t-none-eabi -- {{CLIPPY_ARGUMENTS}})

    mkdir -p examples/target/examples

    arm-none-eabi-objcopy -O binary "$TARGET_FOLDER/thumbv4t-none-eabi/release/$GAME_NAME" "$GBA_FILE"
    gbafix -p "-t${INTERNAL_NAME:0:12}" "-c${INTERNAL_NAME:0:4}" -mGC "$GBA_FILE"

    cp -v "$GBA_FILE" "examples/target/examples/$GAME_NAME.gba"

_all-crates target:
    for CARGO_PROJECT_FILE in agb-*/Cargo.toml agb/Cargo.toml; do \
        PROJECT_DIR=$(dirname "$CARGO_PROJECT_FILE"); \
        just "{{target}}" "$PROJECT_DIR" || exit $?; \
    done

_build-debug crate:
    (cd "{{crate}}" && cargo build --examples --tests)
_build-release crate:
    (cd "{{crate}}" && cargo build --release --examples --tests)
_test-release crate:
    just _build-release {{crate}}
    (cd "{{crate}}" && cargo test --release)
_test-release-arm crate:
    (cd "{{crate}}" && cargo test --release --target="{{justfile_directory()+"/armv4t-none-eabi.json"}}")
_test-debug crate:
    just _build-debug {{crate}}
    (cd "{{crate}}" && cargo test)
_test-debug-arm crate:
    (cd "{{crate}}" && cargo test --release --target="{{justfile_directory()+"/armv4t-none-eabi.json"}}")
_clippy crate:
    (cd "{{crate}}" && cargo clippy --examples --tests -- {{CLIPPY_ARGUMENTS}})
_clean crate:
    (cd "{{crate}}" && cargo clean)

_build-example example:
    (cd agb && cargo build "--example={{example}}")
_build-example-release example:
    (cd agb && cargo build "--example={{example}}" --release)