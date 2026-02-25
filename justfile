# Interactive commands and scripts - build targets are in Makefile
export CARGO_TARGET_DIR := env_var_or_default('CARGO_TARGET_DIR', justfile_directory() + "/target")

podman_command := "podman"

# === CI (delegates to make for parallel builds) ===

# Set LOG_FILE to capture full build output (useful for CI artifact upload)
export LOG_FILE := env_var_or_default('LOG_FILE', '')

ci:
    #!/usr/bin/env bash
    if [ -t 1 ]; then
        # TTY - use TUI tool for nicer interactive experience
        cargo run -p tools --release -q -- build ci
    else
        # Not TTY (CI) - use raw make output
        make -j$(nproc) LOG_FILE="$LOG_FILE" ci
    fi

build:
    make -j$(nproc) build

build-roms:
    make -j$(nproc) build-roms

clippy:
    make -j$(nproc) clippy

fmt-check:
    make -j$(nproc) fmt-check

test:
    make test

test-release:
    make test-release

clean:
    make clean

# === Interactive development commands ===

fmt:
    for CARGO_PROJECT_FILE in agb/Cargo.toml tracker/agb-tracker/Cargo.toml tracker/desktop-player/Cargo.toml ./Cargo.toml; do \
        PROJECT_DIR=$(dirname "$CARGO_PROJECT_FILE"); \
        (cd "$PROJECT_DIR" && cargo fmt --all) || exit $?; \
    done
    for CARGO_PROJECT_FILE in examples/*/Cargo.toml; do \
        PROJECT_DIR=$(dirname "$CARGO_PROJECT_FILE"); \
        (cd "$PROJECT_DIR" && cargo fmt --all) || exit $?; \
    done

run-example example:
    (cd agb && cargo build "--example={{example}}")
    mgba-qt "$CARGO_TARGET_DIR/thumbv4t-none-eabi/debug/examples/{{example}}"

run-example-release example:
    (cd agb && cargo build "--example={{example}}" --release)
    mgba-qt "$CARGO_TARGET_DIR/thumbv4t-none-eabi/release/examples/{{example}}"

run-game game:
    (cd "examples/{{game}}" && cargo run --release)

run-game-debug game:
    (cd "examples/{{game}}" && cargo run)

# === Tools ===

gbafix *args:
    (cd agb-gbafix && cargo build --release && cd "{{invocation_directory()}}" && "$CARGO_TARGET_DIR/release/agb-gbafix" {{args}})

debug *args:
    (cd agb-debug && cargo build --release && cd "{{invocation_directory()}}" && "$CARGO_TARGET_DIR/release/agb-debug" {{args}})

# === Release management ===

publish *args: (_run-tool "publish" args)

release +args: (_run-tool "release" args)

update-lockfiles *args:
    bash .github/scripts/update-lockfiles.sh {{args}}

_run-tool +tool:
    (cd tools && cargo build)
    "$CARGO_TARGET_DIR/debug/tools" {{tool}}

# === Website builds ===

validate-renovate:
    npx --yes --package renovate -- renovate-config-validator

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
    make -j$(nproc) rom-combo
    mkdir -p website/agb/src/roms
    gzip -9 -c examples/target/examples/combo.gba > website/agb/src/roms/combo.gba.gz

build-screenshot-generator:
    (cd emulator/screenshot-generator; cargo build --release)

generate-screenshot *args:
    "$CARGO_TARGET_DIR/release/screenshot-generator" {{args}}

build-site-examples:
    make -j$(nproc) build-site-examples

build-book:
    (cd book && mdbook build)

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

# === Playground ===

build-playground-image:
    (cd website/play/docker && ./build-initial-image.sh)

build-playground-server-image:
    (cd website/play && docker build -t ghcr.io/agbrs/playground-server:latest .)

build-playground-api:
    (cd website/play && cargo build --release --target=x86_64-unknown-linux-musl)

# === Docs ===

check-docs:
    make check-docs

miri:
    make miri

spellcheck:
    make spellcheck
