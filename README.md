# Rust for the Game Boy Advance

This is my in development library for rust on the Game Boy Advance. It uses
information from GbaTek, Tonc, and the existing
[rust-console/gba](https://github.com/rust-console/gba).

Note that this currently contains no documentation of any kind, unless you count
examples as documentation.

## Build Requirements

* Nightly rust, probably quite a recent version.
* arm eabi binutils 
    * Debian and derivatives: `sudo apt install binutils-arm-none-eabi`
    * Arch Linux and derivatives: `pacman -S arm-none-eabi-binutils`

## Test Requirements

* mgba 0.9.0
* [corwinkuiper/mgba-test-runner](https://github.com/corwinkuiper/mgba-test-runner)
    * `cargo install --git https://github.com/corwinkuiper/mgba-test-runner.git`

## Real Hardware Build

* Need gbafix, rust implementation can be installed with `cargo install gbafix`.
* On compiled elf file, additionally need to
```bash
arm-none-eabi-objcopy -O binary {input-elf} {output-gba}
gbafix {output-gba}
```

## Stability

0% stable, I have no problems making drastic changes in the API in order to make
something nice to work with.