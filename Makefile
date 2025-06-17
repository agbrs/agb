CARGO_FLAGS?=--color=always
CLIPPY_FLAGS?=-Dwarnings -Dclippy::all

EXAMPLES_MANIFEST=$(shell find examples -name 'Cargo.toml')
BOOK_EXAMPLES_MANIFEST=$(shell find book -name 'Cargo.toml')
EXAMPLES=$(EXAMPLES_MANIFEST:/Cargo.toml=)
BOOK_EXAMPLES=$(BOOK_EXAMPLES_MANIFEST:/Cargo.toml=)

WORKSPACE=./
TEMPLATE=./template/
TRACKER_EXCLUDES=./tracker/agb-tracker/ ./tracker/desktop-player/

ALL_CRATES=$(EXAMPLES) $(WORKSPACE) $(BOOK_EXAMPLES) $(TEMPLATE) $(TRACKER_EXCLUDES)

export CARGO_TARGET_DIR=${CURDIR}/target

## ALIASES
# These are the main things that should be called

ci: install/test-runner test-all all/fmt-check all/lint spellcheck miri/agb-hashmap doc-check book

test-agb: test/agb release/test/agb arm/test/agb release/arm/test/agb no-features/build/agb

test-native: $(addprefix test/,$(WORKSPACE))

test-all: test-agb test-native

doc-check: thumb/doc/agb thumb/doc/tracker/agb-tracker doc/.

## MODIFIERS
# These are prefixes to rules that affect how the rule is run

all/%: FORCE
	$(MAKE) $(addprefix $*/,$(ALL_CRATES))

release/%: FORCE
	$(MAKE) CARGO_FLAGS="$(CARGO_FLAGS) --release" $*

arm/%: FORCE
	$(MAKE) CARGO_FLAGS="$(CARGO_FLAGS) --target=armv4t-none-eabi" $*

thumb/%: FORCE
	$(MAKE) CARGO_FLAGS="$(CARGO_FLAGS) --target=thumbv4t-none-eabi" $*

no-features/%: FORCE
	$(MAKE) CARGO_FLAGS="$(CARGO_FLAGS) --no-default-features" $*

## CARGO RULES

test/%: install/test-runner
	(cd $* && cargo test $(CARGO_FLAGS))

build/%: FORCE
	(cd $* && cargo build $(CARGO_FLAGS))

lint/%: FORCE
	(cd $* && cargo clippy --examples --tests -- $(CLIPPY_FLAGS))

fmt/%: FORCE
	(cd $* && cargo fmt --all)

fmt-check/%: FORCE
	(cd $* && cargo fmt --all -- --check)

miri/%: install/miri
	(cd $* && cargo miri test --lib $(CARGO_FLAGS))

doc/%: FORCE
	(cd $* && cargo doc --no-deps $(CARGO_FLAGS))

## MISC RULES

spellcheck: FORCE
	npx --yes -- cspell lint '**/*.rs' '**/*.md'

book: FORCE
	(cd book && mdbook build)

## INSTALLS

.PHONY: install/miri install/test-runner install/screenshot-generator

install/miri:
	rustup component add miri clippy rustfmt --toolchain=nightly
	cargo +nightly miri setup

install/test-runner:
	cargo install --path emulator/test-runner --verbose

install/screenshot-generator:
	cargo install --path emulator/screenshot-generator --verbose $(CARGO_FLAGS)

.PHONY: FORCE

FORCE:
