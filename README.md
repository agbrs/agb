# Rust for the Game Boy Advance

This is my in development library for rust on the Game Boy Advance. It uses
information from GbaTek, Tonc, and the existing
[rust-console/gba](https://github.com/rust-console/gba).

Note that this currently contains no documentation of any kind, unless you count
examples as documentation.

## Build Requirements

* Recent rustup, see [the rust website](https://www.rust-lang.org/tools/install)
  for instructions for your operating system.
    * You can update rustup with `rustup update`, or using your package manager
      if you obtained rustup in this way.
* arm eabi binutils 
    * Debian and derivatives: `sudo apt install binutils-arm-none-eabi`
    * Arch Linux and derivatives: `pacman -S arm-none-eabi-binutils`
    * Windows can apparently use the [GNU Arm Embedded
      Toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads).
      Make sure to select "Add path to environment variable" during the install.
    * This process has only been tested on Ubuntu and Arch Linux.

## Test Requirements

* mgba 0.9.X
* mgba-test-runner
    * Run `cargo install --path mgba-test-runner` inside this directory

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