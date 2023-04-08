# AGB

## Rust for the Game Boy Advance

[![Docs](https://docs.rs/agb/badge.svg)](https://docs.rs/agb/latest/agb)
[![Build](https://github.com/agbrs/agb/actions/workflows/build-and-test.yml/badge.svg?branch=master)](https://github.com/agbrs/agb/actions/workflows/build-and-test.yml)
[![Licence](https://img.shields.io/crates/l/agb)](https://www.mozilla.org/en-US/MPL/2.0/)
[![Crates.io](https://img.shields.io/crates/v/agb)](https://crates.io/crates/agb)

![AGB logo](.github/logo.png)

This is a library for making games on the Game Boy Advance using the Rust
programming language. The library's main focus is to provide an abstraction
that allows you to develop games which take advantage of the GBA's capabilities
without needing to have extensive knowledge of its low-level implementation.

agb provides the following features:

* Simple build process with minimal dependencies
* Built in importing of sprites, backgrounds, music and sound effects
* High performance audio mixer
* Easy to use sprite and tiled background usage
* A global allocator allowing for use of both `core` and `alloc`

The documentation for the latest release can be found on
[docs.rs](https://docs.rs/agb/latest/agb/).

## Getting started

The best way to get started with agb is to use the template, either within the
`template` directory in this repository or cloning the [template repository](https://github.com/agbrs/template).

Once you have done this, you will find further instructions within the README in the template.

There is an (in progress) tutorial which you can find on the [project website](https://agbrs.github.io/agb/).

## Help / Support

If you need any help, the [discussions page](https://github.com/agbrs/agb/discussions)
is a great place to get help from the creators and contributors.

Feel free to [create a new discussion in the Q&A category](https://github.com/agbrs/agb/discussions/new?category=Q-A) and we'll do our best to help!


## Contributing to agb itself

In order to contribute to agb itself, you will need a few extra tools on top of what you would need
to just write games for the Game Boy Advance using this library:

* Recent rustup, see [the rust website](https://www.rust-lang.org/tools/install)
  for instructions for your operating system.
    * You can update rustup with `rustup update`, or using your package manager
      if you obtained rustup in this way.
* arm eabi binutils 
    * Debian and derivatives: `sudo apt install binutils-arm-none-eabi`
    * Arch Linux and derivatives: `pacman -S arm-none-eabi-binutils`
    * Windows can apparently use the [GNU Arm Embedded Toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads).
      Make sure to select "Add path to environment variable" during the install.
    * This process has only been tested on Ubuntu and Arch Linux.
* libelf and cmake
  * Debian and derivatives: `sudo apt install libelf-dev cmake`
  * Arch Linux and derivatives: `pacman -S libelf cmake`
* mgba-test-runner
    * Run `cargo install --path mgba-test-runner` inside this directory
* [The 'just' build tool](https://github.com/casey/just)
    * Install with `cargo install just`
* [mdbook](https://rust-lang.github.io/mdBook/index.html)
    * Install with `cargo install mdbook`
* [gbafix](https://crates.io/crates/gbafix)
    * Install with `cargo install gbafix`

With all of this installed, you should be able to run a full build of agb using by running
```sh
just ci
```

Note that before you create a PR, please file an issue so we can discuss what you are looking to change.

## Structure of the repo

`agb-fixnum` - a simple fixed point number storage since the GBA doesn't have a floating point unit, so required
for performant decimals.

`agb-image-converter` - a crate which converts images in normal formats to a format supported by the game boy advance

`agb-macros` - miscellaneous proc-macros which have to be in a different crate

`agb-sound-converter` - a crate which converts wav files into a format supported by the game boy advance

`agb` - the main library code

`agb/examples` - basic examples often targeting 1 feature, you can run these using `just run-example <example-name>`

`book` - the source for the tutorial and website

`book/games` - games made as part of the tutorial

`examples` - bigger examples of a complete game, made during game jams

`mgba-test-runner` - a wrapper around the [mgba](https://mgba.io) emulator which allows us to write unit tests in rust

`template` - the source for the [template repository](https://github.com/agbrs/template)

## Stability

While agb is in the pre-1.0 phase, we follow a semi-semantic versioning scheme to ensure compatibility between minor releases.
Specifically, any 0.x.y release is guaranteed to be compatible with another 0.x.z release provided that y > z, but there may be breaking changes between minor releases (i.e., changes to the second digit, e.g., between 0.1 and 0.2).

Once agb reaches version 1.0, we will transition to stronger semantic versioning, meaning that any breaking changes will be indicated by an increment to the major version (i.e., the first digit, e.g., from 1.0 to 2.0).

## Acknowledgments

agb would not be possible without the help from the following (non-exhaustive) list of projects:

* The amazing work of the [rust-console](https://github.com/rust-console) for making this all possible in the first place
* The [asefile](https://crates.io/crates/asefile) crate for loading aseprite files
* [agbabi](https://github.com/felixjones/agbabi) for providing high performance alternatives to common methods
* [mgba](https://mgba.io) for all the useful debugging / developer tools built in to the emulator

## Licence

agb and all its subcrates (except agb-gbafix) are released under MPL version 2.0. See full licence
text in the `LICENSE` file.

agb-gbafix is released under GPL version 3.0. See the full licence in the agb-gbafix/LICENSE file

agb contains a subset of the code from [agbabi](https://github.com/felixjones/agbabi) which is released under a zlib style licence,
details for which you can find under `agb/src/agbabi`.

The template is released under [CC0](https://creativecommons.org/share-your-work/public-domain/cc0/) to allow you to make whatever
changes you wish.

The agb logo is released under [Creative Commons Attribution-ShareAlike 4.0](http://creativecommons.org/licenses/by-sa/4.0/)

The music used for the examples is by [Josh Woodward](https://www.joshwoodward.com) and released under [Creative Commons Attribution 4.0](https://creativecommons.org/licenses/by/4.0/)