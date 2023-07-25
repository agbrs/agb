export CARGO_TARGET_DIR := env_var_or_default('CARGO_TARGET_DIR', justfile_directory() + "/target")
CLIPPY_ARGUMENTS := "-Dwarnings -Dclippy::all -Aclippy::empty-loop"

build: build-roms

build-debug:
    just _build-debug agb
    just _build-debug tracker/agb-tracker
build-release:
    just _build-release agb
    just _build-release tracker/agb-tracker
clippy:
    just _all-crates _clippy
    just _clippy tools

test:
    just _test-debug agb
    just _test-debug agb-fixnum
    just _test-debug agb-hashmap
    just _test-debug tracker/agb-tracker
    just _test-debug-arm agb
    just _test-debug tools

test-release:
    just _test-release agb
    just _test-release agb-fixnum
    just _test-release tracker/agb-tracker
    just _test-release-arm agb

doctest-agb:
    (cd agb && cargo test --doc -Z doctest-xcompile)

check-docs:
    (cd agb && cargo doc --target=thumbv4t-none-eabi --no-deps)
    (cd tracker/agb-tracker && cargo doc --target=thumbv4t-none-eabi --no-deps)
    just _build_docs agb-fixnum
    just _build_docs agb-hashmap

_build_docs crate:
    (cd "{{crate}}" && cargo doc --no-deps)

clean:
    just _all-crates _clean

fmt:
    just _all-crates _fmt
    just _fmt tools
fmt-check:
    just _all-crates _fmt-check
    just _fmt-check tools

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

check-linker-script-consistency:
    find -type f -name gba.ld -print0 | xargs -0 -n1 cmp -- agb/gba.ld
    find -type f -name gba_mb.ld -print0 | xargs -0 -n1 cmp -- agb/gba_mb.ld

ci: check-linker-script-consistency build-debug clippy fmt-check test miri build-release test-release doctest-agb build-roms build-book check-docs

build-roms:
    just _build-rom "examples/the-purple-night" "PURPLENIGHT"
    just _build-rom "examples/the-hat-chooses-the-wizard" "HATWIZARD"
    just _build-rom "examples/hyperspace-roll" "HYPERSPACE"
    just _build-rom "examples/the-dungeon-puzzlers-lament" "DUNGLAMENT"
    just _build-rom "examples/amplitude" "AMPLITUDE"
    just _build-rom "examples/combo" "AGBGAMES"

    just _build-rom "book/games/pong" "PONG"

    (cd examples/target && zip examples.zip examples/*.gba)

build-book:
    (cd book && mdbook build)

update-lockfiles:
    bash .github/scripts/update-lockfiles.sh

update-linker-scripts:
    find -type f -name gba.ld | grep -v ./agb/gba.ld | xargs -n1 cp -v -- agb/gba.ld
    find -type f -name gba_mb.ld | grep -v ./agb/gba_mb.ld | xargs -n1 cp -v -- agb/gba_mb.ld

publish *args: (_run-tool "publish" args)

release +args: (_run-tool "release" args)

miri:
    (cd agb-hashmap && cargo miri test)

_run-tool +tool:
    (cd tools && cargo build)
    "$CARGO_TARGET_DIR/debug/tools" {{tool}}

_build-rom folder name:
    #!/usr/bin/env bash
    set -euxo pipefail

    GAME_FOLDER="{{folder}}"
    INTERNAL_NAME="{{name}}"

    GAME_NAME="$(basename "$GAME_FOLDER")"

    TARGET_FOLDER="${CARGO_TARGET_DIR:-$GAME_FOLDER/target}"
    GBA_FILE="$TARGET_FOLDER/$GAME_NAME.gba"

    (cd "$GAME_FOLDER" && cargo build --release --target thumbv4t-none-eabi && cargo clippy --release --target thumbv4t-none-eabi -- {{CLIPPY_ARGUMENTS}} && cargo fmt --all -- --check)

    mkdir -p examples/target/examples

    just gbafix --title "${INTERNAL_NAME:0:12}" --gamecode "${INTERNAL_NAME:0:4}" --makercode GC "$TARGET_FOLDER/thumbv4t-none-eabi/release/$GAME_NAME" -o "$GBA_FILE"

    cp -v "$GBA_FILE" "examples/target/examples/$GAME_NAME.gba"

gbafix *args:
    (cd agb-gbafix && cargo run --release -- {{args}})

_all-crates target:
    for CARGO_PROJECT_FILE in agb-*/Cargo.toml agb/Cargo.toml tracker/agb-*/Cargo.toml; do \
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
    (cd "{{crate}}" && cargo test --release --target=armv4t-none-eabi)
_test-debug crate:
    just _build-debug {{crate}}
    (cd "{{crate}}" && cargo test)
_test-debug-arm crate:
    (cd "{{crate}}" && cargo test --target=armv4t-none-eabi)
_clippy crate:
    (cd "{{crate}}" && cargo clippy --examples --tests -- {{CLIPPY_ARGUMENTS}})
_clean crate:
    (cd "{{crate}}" && cargo clean)
_fmt crate:
    (cd "{{crate}}" && cargo fmt --all)
_fmt-check crate:
    (cd "{{crate}}" && cargo fmt --all -- --check)

_build-example example:
    (cd agb && cargo build "--example={{example}}")
_build-example-release example:
    (cd agb && cargo build "--example={{example}}" --release)
