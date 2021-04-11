# Rust for the Gameboy Advance

This is my in development library for rust on the gameboy advance. It uses
information from GbaTek, Tonc, and the existing
[rust-console/gba](https://github.com/rust-console/gba).

Note that this currently contains no documentation of any kind, unless you count
examples as documentation.

## Build Requirements

* Nightly rust, probably quite a recent version.
* arm eabi binutils 
    * Debian and derivatives: binutils-arm-none-eabi
    * Alpine: binutils-arm-none-eabi
    * Arch Linux and derivatives: arm-none-eabi-binutils

## Test Requirements

* need a recent mgba (known to work on 0.8.4+)
* xvfb
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