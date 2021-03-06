BINUTILS_PREFIX=arm-none-eabi-
CC=$(BINUTILS_PREFIX)as
ARCH = -mthumb-interwork -mthumb

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
	@mgba-qt $<
	@rm -f out/debug/$${OUTNAME}.sav

r-%: out/release/%.gba
	@OUTNAME=$(patsubst r-%,%,$@)
	@mgba-qt $<
	@rm -f out/release/$${OUTNAME}.sav
	
cargo-release-%: $(RUSTFILES) out/crt0.o
	@OUTNAME=$(patsubst cargo-release-%,%, $@)
	@rustup run nightly cargo xbuild --release --target=gba.json --example=$${OUTNAME}

cargo-debug-%: $(RUSTFILES) out/crt0.o
	@OUTNAME=$(patsubst cargo-debug-%,%, $@)
	@rustup run nightly cargo xbuild --target=gba.json --example=$${OUTNAME}

out/crt0.o: crt0.s interrupt_simple.s
	@mkdir $(dir $@)
	@$(CC) $(ARCH) -o out/crt0.o crt0.s

clippy:
	rustup run nightly cargo xclippy --target=gba.json