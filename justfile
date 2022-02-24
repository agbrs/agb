build: _build-roms

ci:
    just _all-crates _build
    just _all-crates _test-debug
    just _all-crates _test-release
    just _all-crates _clippy

_build-roms:
    just _build-rom "examples/the-purple-night" "PURPLENIGHT"
    just _build-rom "examples/the-hat-chooses-the-wizard" "HATWIZARD"

    just _build-rom "book/games/pong" "PONG"

    (cd examples/target && zip examples.zip examples/*.gba)

_build-rom folder name:
    #!/usr/bin/env bash
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
        just "{{target}}" "$PROJECT_DIR"; \
    done

_build crate:
    (cd "{{crate}}" && cargo build)
_test-release crate:
    if echo "{{crate}}" | grep -qE '^agb'; then (cd "{{crate}}" && cargo test --release); fi
_test-debug crate:
    if echo "{{crate}}" | grep -qE '^agb'; then (cd "{{crate}}" && cargo test); fi
_clippy crate:
    if echo "{{crate}}" | grep -qE '^agb'; then (cd "{{crate}}" && cargo clippy); fi
