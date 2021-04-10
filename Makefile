RUSTFILES=$(shell find . -name '*.rs')

.ONESHELL:

out/debug/%.gba: cargo-debug-%
	@mkdir -p $(dir $@)
	@OUTNAME=$(patsubst out/debug/%.gba,%,$@)
	@$(BINUTILS_PREFIX)objcopy -O binary target/gba/debug/examples/$${OUTNAME} out/debug/$${OUTNAME}.gba
	@gbafix $@

out/release/%.gba: cargo-release-%
	@mkdir -p $(dir $@)
	@OUTNAME=$(patsubst out/release/%.gba,%,$@)
	@$(BINUTILS_PREFIX)objcopy -O binary target/gba/release/examples/$${OUTNAME} out/release/$${OUTNAME}.gba
	@gbafix $@

d-%: out/debug/%.gba
	@OUTNAME=$(patsubst d-%,%,$@)
	@mgba-qt -l 31 -d -C logToStdout=1 $<
	@rm -f out/debug/$${OUTNAME}.sav

r-%: out/release/%.gba
	@OUTNAME=$(patsubst r-%,%,$@)
	@mgba-qt -l 31 -d -C logToStdout=1 $<
	@rm -f out/release/$${OUTNAME}.sav
	
cargo-release-%: $(RUSTFILES)
	@OUTNAME=$(patsubst cargo-release-%,%, $@)
	@rustup run nightly cargo build --release --example=$${OUTNAME}

cargo-debug-%: $(RUSTFILES)
	@OUTNAME=$(patsubst cargo-debug-%,%, $@)
	@rustup run nightly cargo build --example=$${OUTNAME}

doc:
	rustup run nightly cargo doc