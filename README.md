# Rust for the Game Boy Advance

![AGB logo](.github/logo.png)

This is a library for making games on the Game Boy Advance using the Rust
programming language. It attempts to be a high level abstraction over the
internal workings of the Game Boy Advance whilst still being high performance
and memory efficient.

The documentation for the latest release can be found on
[docs.rs](https://docs.rs/agb/latest/agb/). Note that this repository does not
nesesarily contain the latest release, but in development versions. Futher work
is needed to improve the documentation.


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

* libelf
  * Debian and derivatives: `sudo apt install libelf-dev`
  * Arch Linux and derivatives: `pacman -S libelf`
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