export CARGO_TARGET_DIR := env_var_or_default('CARGO_TARGET_DIR', justfile_directory() + "/target")
CLIPPY_ARGUMENTS := "-Dwarnings -Dclippy::all"

podman_command := "podman"

build: build-roms

build-debug:
    (cd agb && cargo build --no-default-features)
    (cd agb && cargo build --no-default-features --features=testing)
    (cd agb && cargo build --examples --tests)

    (cd tracker/agb-tracker && cargo build --examples --tests)

build-release:
    (cd agb && cargo build --examples --tests --release)

clippy:
    just _all-crates _clippy

test:
    # test the workspace
    cargo test
    # also need to explicitly hit the serde tests in agb-hashmap
    (cd agb-hashmap && cargo test --features=serde serde)
    just _test-debug agb
    just _test-debug tracker/agb-tracker
    just _test-multiboot
    just _test-debug-arm agb

test-release:
    just _test-release agb
    just _test-release tracker/agb-tracker
    just _test-release-arm agb

check-docs:
    (cd agb && cargo doc --target=thumbv4t-none-eabi --no-deps)
    (cd tracker/agb-tracker && cargo doc --target=thumbv4t-none-eabi --no-deps)
    cargo doc --no-deps

validate-renovate:
    npx --yes --package renovate -- renovate-config-validator

_build_docs crate:
    (cd "{{crate}}" && cargo doc --no-deps)

clean:
    just _all-crates _clean

fmt:
    just _all-crates _fmt
    just _all-examples _fmt

fmt-check:
    just _all-crates _fmt-check
    just _all-examples _fmt-check

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

ci: build-debug clippy fmt-check spellcheck test miri build-release test-release build-roms build-book check-docs

build-roms:
    just _build-rom "examples/the-purple-night" "PURPLENIGHT"
    just _build-rom "examples/the-hat-chooses-the-wizard" "HATWIZARD"
    just _build-rom "examples/hyperspace-roll" "HYPERSPACE"
    just _build-rom "examples/the-dungeon-puzzlers-lament" "DUNGLAMENT"
    just _build-rom "examples/amplitude" "AMPLITUDE"
    just _build-rom "examples/combo" "AGBGAMES"

    just _build-rom "book/games/pong" "PONG"
    just _build-rom "book/games/platform" "PLATFORM"

    (cd examples/target && zip examples.zip examples/*.gba)

build-book:
    (cd book && mdbook build)

update-lockfiles *args:
    bash .github/scripts/update-lockfiles.sh {{args}}

publish *args: (_run-tool "publish" args)

release +args: (_run-tool "release" args)

miri:
    (cd agb-hashmap && cargo miri test --lib)

setup-cargo-wasm:
    cargo install wasm-pack

build-website-backtrace:
    (cd website/backtrace && wasm-pack build --target web)
    rm -rf website/agb/src/vendor/backtrace
    mkdir -p website/agb/src/vendor
    cp website/backtrace/pkg website/agb/src/vendor/backtrace -r

build-mgba-wasm:
    rm -rf website/agb/src/components/mgba/vendor
    mkdir website/agb/src/components/mgba/vendor
    {{podman_command}} build --file website/mgba-wasm/BuildMgbaWasm --output=website/agb/src/components/mgba/vendor .

build-combo-rom-site:
    just _build-rom "examples/combo" "AGBGAMES"
    mkdir -p website/agb/src/roms
    gzip -9 -c examples/target/examples/combo.gba > website/agb/src/roms/combo.gba.gz

build-screenshot-generator:
    (cd emulator/screenshot-generator; cargo build --release)

generate-screenshot *args:
    "$CARGO_TARGET_DIR/release/screenshot-generator" {{args}}


build-site-examples: build-release
    #!/usr/bin/env bash
    set -euxo pipefail

    mkdir -p website/agb/src/roms/examples

    EXAMPLES="$(cd agb/examples; ls *.rs)"
    EXAMPLE_DEFINITIONS="export const Examples: {url: URL, example_name: string, screenshot: StaticImageData }[] = [" > website/agb/src/roms/examples/examples.ts
    EXAMPLE_IMAGE_IMPORTS="import { StaticImageData } from 'next/image';";

    for EXAMPLE_NAME in $EXAMPLES; do
        EXAMPLE="${EXAMPLE_NAME%.rs}"
        just gbafix "$CARGO_TARGET_DIR/thumbv4t-none-eabi/release/examples/$EXAMPLE" --output="$CARGO_TARGET_DIR/thumbv4t-none-eabi/release/examples/$EXAMPLE.gba"
        cp "agb/examples/$EXAMPLE_NAME" "website/agb/src/roms/examples/$EXAMPLE_NAME"
        gzip -9 -c $CARGO_TARGET_DIR/thumbv4t-none-eabi/release/examples/$EXAMPLE.gba > website/agb/src/roms/examples/$EXAMPLE.gba.gz
        just generate-screenshot --rom="$CARGO_TARGET_DIR/thumbv4t-none-eabi/release/examples/$EXAMPLE.gba" --frames=100 --output=website/agb/src/roms/examples/$EXAMPLE.png
        EXAMPLE_IMAGE_IMPORTS="$EXAMPLE_IMAGE_IMPORTS import $EXAMPLE from './$EXAMPLE.png';"
        EXAMPLE_DEFINITIONS="$EXAMPLE_DEFINITIONS {url: new URL('./$EXAMPLE.gba.gz', import.meta.url), example_name: '$EXAMPLE', screenshot: $EXAMPLE},"
    done

    EXAMPLE_DEFINITIONS="$EXAMPLE_DEFINITIONS ];"
    echo "$EXAMPLE_IMAGE_IMPORTS" > website/agb/src/roms/examples/examples.ts
    echo "$EXAMPLE_DEFINITIONS" >> website/agb/src/roms/examples/examples.ts

build-site-dependencies: build-combo-rom-site build-site-examples build-book

package-site-dependencies: build-site-dependencies
    mkdir -p target
    tar -cf target/site-deps.tar.gz book/book website/agb/src/roms/examples website/agb/src/roms/combo.gba.gz

unpackage-site-dependencies:
    tar -xvf target/site-deps.tar.gz

setup-app-build: build-mgba-wasm build-website-backtrace
    (cd website/agb && npm install --no-save --prefer-offline --no-audit)

_build-site-app: setup-app-build
    (cd website/agb && npm run build)

serve-site-dev: build-screenshot-generator build-site-dependencies setup-app-build
    (cd website/agb && npm run dev)

_build-site-ci: unpackage-site-dependencies _build-site-app
    rm -rf website/build
    cp website/agb/out website/build -r
    cp book/book website/build/book -r

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
    (cd agb-gbafix && cargo build --release && cd "{{invocation_directory()}}" && "$CARGO_TARGET_DIR/release/agb-gbafix" {{args}})

debug *args:
    (cd agb-debug && cargo build --release && cd "{{invocation_directory()}}" && "$CARGO_TARGET_DIR/release/agb-debug" {{args}})

_all-crates target:
    for CARGO_PROJECT_FILE in agb/Cargo.toml tracker/agb-tracker/Cargo.toml tracker/desktop-player/Cargo.toml ./Cargo.toml; do \
        PROJECT_DIR=$(dirname "$CARGO_PROJECT_FILE"); \
        just "{{target}}" "$PROJECT_DIR" || exit $?; \
    done

_all-examples target:
    for CARGO_PROJECT_FILE in examples/*/Cargo.toml; do \
        PROJECT_DIR=$(dirname "$CARGO_PROJECT_FILE"); \
        just "{{target}}" "$PROJECT_DIR" || exit $?; \
    done

build-playground-image:
    (cd website/play/docker && ./build-initial-image.sh)

build-playground-server-image:
    (cd website/play && docker build -t ghcr.io/agbrs/playground-server:latest .)

build-playground-api:
    (cd website/play && cargo build --release --target=x86_64-unknown-linux-musl)

spellcheck:
    npx --yes -- cspell lint '**/*.rs' '**/*.md' 

_test-release crate:
    (cd "{{crate}}" && cargo test --release)
_test-release-arm crate:
    (cd "{{crate}}" && cargo test --release --target=armv4t-none-eabi)
_test-debug crate:
    (cd "{{crate}}" && cargo test)
_test-debug-arm crate:
    (cd "{{crate}}" && cargo test --target=armv4t-none-eabi)
_test-multiboot:
    (cd "agb" && AGB_MULTIBOOT=true cargo test --features=multiboot --test=test_multiboot)
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
