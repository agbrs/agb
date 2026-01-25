.PHONY: all build ci clean build-debug build-release clippy fmt-check test test-release \
        miri build-roms build-book check-docs spellcheck build-site-examples \
        $(ROM_TARGETS) $(CLIPPY_TARGETS) $(FMT_CHECK_TARGETS) $(FMT_CHECK_EXAMPLE_TARGETS)

SHELL := /bin/bash
.ONESHELL:

export CARGO_TARGET_DIR ?= $(CURDIR)/target
CLIPPY_ARGUMENTS := -Dwarnings -Dclippy::all

# Optional log file for full output (useful for CI artifact upload)
# Usage: make LOG_FILE=build.log ci
LOG_FILE ?=

# Quiet command runner - shows message, only prints output on error
# Usage: $(call run,Message to display,command to run)
# Prints complete line atomically to avoid interleaving in parallel builds
# Filters out known-benign warnings (assembler register order, cargo lock messages)
# Command runs in a subshell to avoid .ONESHELL issues with cd
# If LOG_FILE is set, appends full output to that file (atomically via flock)
define run
	@printf "  \033[33m▶\033[0m %s\n" "$(1)"; \
	tmpfile=$$(mktemp); \
	($(2)) > "$$tmpfile" 2>&1; rc=$$?; \
	if [ -n "$(LOG_FILE)" ]; then \
		{ printf '\n=== %s ===\n' "$(1)"; cat "$$tmpfile"; } | flock "$(LOG_FILE).lock" tee -a "$(LOG_FILE)" > /dev/null; \
	fi; \
	if [ $$rc -eq 0 ]; then \
		printf "  \033[32m✓\033[0m %s\n" "$(1)"; \
	else \
		printf "  \033[31m✗\033[0m %s\n" "$(1)"; \
		grep -v -E "(register list not in ascending order|Blocking waiting for file lock|instantiated into assembly here|while in macro instantiation|^\s*-->|^\s*\|$$|^\s*\^$$|^note:)" "$$tmpfile" || true; \
		rm -f "$$tmpfile"; \
		exit $$rc; \
	fi; \
	rm -f "$$tmpfile"
endef

# Crates to run clippy/fmt on
CRATES := agb tracker/agb-tracker tracker/desktop-player .

# agb/examples/*.rs files for site examples (discover dynamically)
SITE_EXAMPLES := $(basename $(notdir $(wildcard agb/examples/*.rs)))

# Example games (directories with Cargo.toml, excluding examples-save which is just for shared saves)
EXAMPLE_GAMES := amplitude combo dynamic-isometric hyperspace-roll \
                 the-dungeon-puzzlers-lament the-hat-chooses-the-wizard the-purple-night

# ROMs to build with their internal names
# Format: folder:internal_name
ROMS := \
    examples/the-purple-night:PURPLENIGHT \
    examples/the-hat-chooses-the-wizard:HATWIZARD \
    examples/hyperspace-roll:HYPERSPACE \
    examples/the-dungeon-puzzlers-lament:DUNGLAMENT \
    examples/amplitude:AMPLITUDE \
    examples/dynamic-isometric:ISOMETRIC \
    examples/combo:AGBGAMES \
    book/games/pong:PONG \
    book/games/platform:PLATFORM

# Generate target names from ROM list
ROM_TARGETS := $(foreach rom,$(ROMS),rom-$(notdir $(firstword $(subst :, ,$(rom)))))

# Generate clippy targets
CLIPPY_TARGETS := $(foreach crate,$(CRATES),clippy-$(subst /,-,$(subst .,-root,$(crate))))

# Generate fmt-check targets for crates and examples
FMT_CHECK_TARGETS := $(foreach crate,$(CRATES),fmt-check-$(subst /,-,$(subst .,-root,$(crate))))
FMT_CHECK_EXAMPLE_TARGETS := $(foreach game,$(EXAMPLE_GAMES),fmt-check-example-$(game))

all: build

build: build-roms

ci: build-debug clippy fmt-check spellcheck test miri build-release test-release build-roms build-book check-docs

# === Build targets ===

build-debug:
	$(call run,agb build (no-default-features),cd agb && cargo build -q --no-default-features)
	$(call run,agb build (testing),cd agb && cargo build -q --no-default-features --features=testing)
	$(call run,agb build (examples/tests),cd agb && cargo build -q --examples --tests)
	$(call run,agb-tracker build,cd tracker/agb-tracker && cargo build -q --examples --tests)

build-release:
	$(call run,agb build release,cd agb && cargo build -q --examples --tests --release)

# === Clippy (parallel across crates) ===

clippy: $(CLIPPY_TARGETS)

clippy--root:
	$(call run,clippy (workspace),cargo clippy -q --examples --tests -- $(CLIPPY_ARGUMENTS))

clippy-agb:
	$(call run,clippy agb,cd agb && cargo clippy -q --examples --tests -- $(CLIPPY_ARGUMENTS))

clippy-tracker-agb-tracker:
	$(call run,clippy agb-tracker,cd tracker/agb-tracker && cargo clippy -q --examples --tests -- $(CLIPPY_ARGUMENTS))

clippy-tracker-desktop-player:
	$(call run,clippy desktop-player,cd tracker/desktop-player && cargo clippy -q --examples --tests -- $(CLIPPY_ARGUMENTS))

# === Format check (parallel across crates and examples) ===

fmt-check: $(FMT_CHECK_TARGETS) $(FMT_CHECK_EXAMPLE_TARGETS)

fmt-check--root:
	$(call run,fmt-check (workspace),cargo fmt --all -- --check)

fmt-check-agb:
	$(call run,fmt-check agb,cd agb && cargo fmt -- --check)

fmt-check-tracker-agb-tracker:
	$(call run,fmt-check agb-tracker,cd tracker/agb-tracker && cargo fmt -- --check)

fmt-check-tracker-desktop-player:
	$(call run,fmt-check desktop-player,cd tracker/desktop-player && cargo fmt -- --check)

# Example fmt-check targets (generated pattern)
$(FMT_CHECK_EXAMPLE_TARGETS): fmt-check-example-%:
	$(call run,fmt-check example/$*,cd examples/$* && cargo fmt -- --check)

# === Tests ===
# Note: can't use -q with cargo test as it passes --quiet to the test runner

test:
	$(call run,test (workspace),cargo test)
	$(call run,test agb-hashmap (serde),cd agb-hashmap && cargo test --features=serde serde)
	$(call run,test agb,cd agb && cargo test)
	$(call run,test agb-tracker,cd tracker/agb-tracker && cargo test)
	$(call run,test agb (multiboot),cd agb && AGB_MULTIBOOT=true cargo test --features=multiboot --test=test_multiboot)
	$(call run,test agb (arm),cd agb && cargo test --target=armv4t-none-eabi)

test-release:
	$(call run,test agb (release),cd agb && cargo test --release)
	$(call run,test agb-tracker (release),cd tracker/agb-tracker && cargo test --release)
	$(call run,test agb (release arm),cd agb && cargo test --release --target=armv4t-none-eabi)

# === Other CI targets ===

miri:
	$(call run,miri agb-hashmap,cd agb-hashmap && cargo miri test --lib)

spellcheck:
	$(call run,spellcheck,npx --yes -- cspell lint '**/*.rs' '**/*.md')

check-docs:
	$(call run,docs agb,cd agb && cargo doc -q --target=thumbv4t-none-eabi --no-deps)
	$(call run,docs agb-tracker,cd tracker/agb-tracker && cargo doc -q --target=thumbv4t-none-eabi --no-deps)
	$(call run,docs (workspace),cargo doc -q --no-deps)

build-book:
	$(call run,build book,cd book && mdbook build)

# === Tools ===
# We always invoke cargo and let it decide whether to rebuild.
# Cargo is fast at determining "nothing to do" (~50ms).
# The FORCE dependency ensures Make always runs the recipe, but downstream
# targets only rebuild if the binary timestamp actually changes.

GBAFIX := $(CARGO_TARGET_DIR)/release/agb-gbafix
SCREENSHOT_GENERATOR := $(CARGO_TARGET_DIR)/release/screenshot-generator

.PHONY: FORCE
FORCE:

$(GBAFIX): FORCE
	$(call run,build gbafix,cd agb-gbafix && cargo build -q --release)

$(SCREENSHOT_GENERATOR): FORCE
	$(call run,build screenshot-generator,cd emulator/screenshot-generator && cargo build -q --release)

# === Site examples (parallel gbafix/gzip/screenshot per example) ===

SITE_EXAMPLE_DIR := website/agb/src/roms/examples

# The .gba files we produce for each example
SITE_EXAMPLE_GBAS := $(addprefix $(SITE_EXAMPLE_DIR)/,$(addsuffix .gba,$(SITE_EXAMPLES)))
SITE_EXAMPLE_GZS := $(addsuffix .gz,$(SITE_EXAMPLE_GBAS))
SITE_EXAMPLE_PNGS := $(addprefix $(SITE_EXAMPLE_DIR)/,$(addsuffix .png,$(SITE_EXAMPLES)))
SITE_EXAMPLE_SOURCES := $(addprefix $(SITE_EXAMPLE_DIR)/,$(addsuffix .rs,$(SITE_EXAMPLES)))

# Main target: build all site examples and generate TypeScript
build-site-examples: build-release $(GBAFIX) $(SCREENSHOT_GENERATOR) $(SITE_EXAMPLE_GBAS) $(SITE_EXAMPLE_GZS) $(SITE_EXAMPLE_PNGS) $(SITE_EXAMPLE_SOURCES)
	$(call generate-site-examples-ts)

# Ensure output directory exists
$(SITE_EXAMPLE_DIR):
	mkdir -p $@

# Pattern rule: create .gba from built example (gbafix)
$(SITE_EXAMPLE_DIR)/%.gba: $(CARGO_TARGET_DIR)/thumbv4t-none-eabi/release/examples/% $(GBAFIX) | $(SITE_EXAMPLE_DIR)
	$(call run,gbafix $*,$(GBAFIX) $< --output=$@)

# Pattern rule: gzip the .gba
$(SITE_EXAMPLE_DIR)/%.gba.gz: $(SITE_EXAMPLE_DIR)/%.gba
	$(call run,gzip $*,gzip -9 -c $< > $@)

# Pattern rule: generate screenshot from .gba
$(SITE_EXAMPLE_DIR)/%.png: $(SITE_EXAMPLE_DIR)/%.gba $(SCREENSHOT_GENERATOR)
	$(call run,screenshot $*,$(SCREENSHOT_GENERATOR) --rom=$< --frames=100 --output=$@)

# Pattern rule: copy source .rs file
$(SITE_EXAMPLE_DIR)/%.rs: agb/examples/%.rs | $(SITE_EXAMPLE_DIR)
	$(call run,copy $*.rs,cp $< $@)

# Generate the TypeScript file that imports all examples
define generate-site-examples-ts
	@echo "Generating examples.ts..."
	@{
		echo "import { StaticImageData } from 'next/image';"
		for ex in $(SITE_EXAMPLES); do \
			echo "import $$ex from './$$ex.png';"; \
		done
		echo ""
		echo "export const Examples: {url: URL, example_name: string, screenshot: StaticImageData }[] = ["
		for ex in $(SITE_EXAMPLES); do \
			echo "  {url: new URL('./$$ex.gba.gz', import.meta.url), example_name: '$$ex', screenshot: $$ex},"; \
		done
		echo "];"
	} > $(SITE_EXAMPLE_DIR)/examples.ts
endef

# === ROM builds (parallel) ===

build-roms: $(GBAFIX) $(ROM_TARGETS)
	$(call run,zip roms,mkdir -p examples/target/examples && cd examples/target && (zip -f examples.zip examples/*.gba 2>/dev/null || zip examples.zip examples/*.gba))

# Individual ROM targets (depend on gbafix tool)
rom-the-purple-night: $(GBAFIX)
	$(call build-rom,examples/the-purple-night,PURPLENIGHT)

rom-the-hat-chooses-the-wizard: $(GBAFIX)
	$(call build-rom,examples/the-hat-chooses-the-wizard,HATWIZARD)

rom-hyperspace-roll: $(GBAFIX)
	$(call build-rom,examples/hyperspace-roll,HYPERSPACE)

rom-the-dungeon-puzzlers-lament: $(GBAFIX)
	$(call build-rom,examples/the-dungeon-puzzlers-lament,DUNGLAMENT)

rom-amplitude: $(GBAFIX)
	$(call build-rom,examples/amplitude,AMPLITUDE)

rom-dynamic-isometric: $(GBAFIX)
	$(call build-rom,examples/dynamic-isometric,ISOMETRIC)

rom-combo: $(GBAFIX)
	$(call build-rom,examples/combo,AGBGAMES)

rom-pong: $(GBAFIX)
	$(call build-rom,book/games/pong,PONG)

rom-platform: $(GBAFIX)
	$(call build-rom,book/games/platform,PLATFORM)

# ROM build function
# $(1) = folder, $(2) = internal name
define build-rom
	@printf "  \033[33m▶\033[0m %s\n" "rom $(notdir $(1))"; \
	GAME_FOLDER="$(1)"; \
	INTERNAL_NAME="$(2)"; \
	GAME_NAME="$$(basename "$$GAME_FOLDER")"; \
	TARGET_FOLDER="$(CARGO_TARGET_DIR)"; \
	GBA_FILE="$$TARGET_FOLDER/$$GAME_NAME.gba"; \
	ROM_OUTPUT_DIR="$(CURDIR)/examples/target/examples"; \
	tmpfile=$$(mktemp); \
	( \
		cd "$$GAME_FOLDER" && \
		cargo build -q --release --target thumbv4t-none-eabi && \
		cargo clippy -q --release --target thumbv4t-none-eabi -- $(CLIPPY_ARGUMENTS) && \
		cargo fmt --all -- --check && \
		mkdir -p "$$ROM_OUTPUT_DIR" && \
		$(GBAFIX) --title "$${INTERNAL_NAME:0:12}" --gamecode "$${INTERNAL_NAME:0:4}" --makercode GC \
			"$$TARGET_FOLDER/thumbv4t-none-eabi/release/$$GAME_NAME" -o "$$GBA_FILE" && \
		cp "$$GBA_FILE" "$$ROM_OUTPUT_DIR/$$GAME_NAME.gba" \
	) > "$$tmpfile" 2>&1; rc=$$?; \
	if [ -n "$(LOG_FILE)" ]; then \
		{ printf '\n=== rom %s ===\n' "$(notdir $(1))"; cat "$$tmpfile"; } | flock "$(LOG_FILE).lock" tee -a "$(LOG_FILE)" > /dev/null; \
	fi; \
	filtered=$$(grep -v -E "(register list not in ascending order|Blocking waiting for file lock|instantiated into assembly here|while in macro instantiation|^\s*-->|^\s*\|$$|^\s*\^$$|^note:)" "$$tmpfile" || true); \
	rm -f "$$tmpfile"; \
	if [ $$rc -eq 0 ]; then \
		printf "  \033[32m✓\033[0m %s\n" "rom $(notdir $(1))"; \
	else \
		printf "  \033[31m✗\033[0m %s\n" "rom $(notdir $(1))"; \
		printf '%s\n' "$$filtered"; \
		exit $$rc; \
	fi
endef

# === Clean ===

clean:
	$(call run,clean workspace,cargo clean -q)
	$(call run,clean agb,cd agb && cargo clean -q)
	$(call run,clean agb-tracker,cd tracker/agb-tracker && cargo clean -q)
	$(call run,clean desktop-player,cd tracker/desktop-player && cargo clean -q)
