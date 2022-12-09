# Mac setup

This guide has been tested on MacOS 13.0.1 on an M1 chip.

# 1. Install a recent version of rust

agb unfortunately relies on a few nightly rust features, so you need to ensure you have that installed.
Firstly, ensure that you have **rustup** installed which you can do by following the instructions on the [rust website](https://www.rust-lang.org/tools/install)

You can update rustup with `rustup update` if you have already installed it.

# 2. Install arm-none-eabi

We need this installed in order to be able to assemble the small amount of assembly in agb, and to do the final linking.

## Install from ARM

Download the toolchain from [ARM here](https://developer.arm.com/downloads/-/gnu-rm)
 * Run the .pkg to install
 * Add `/Applications/ARM/bin` to your `/etc/paths` file

## Install from Homebrew

Or you can try installing with homebrew from the [Arm Mbed repo](https://github.com/ARMmbed/homebrew-formulae):

```
brew tap ArmMbed/homebrew-formulae
brew install arm-none-eabi-gcc
```

# 3. Get git

The source code for the game is hosted on github, so you will need git installed. Follow the instructions at [git-scm.com](https://git-scm.com/)

# 4. GBA Emulator - mGBA

We recommend using the mGBA emulator which you can download for Mac [here](https://mgba.io/downloads.html).

After installing to your `/Applications` folder you can add the binary to your path and create an alias for the agb run command to use.

* Add `/Applications/mGBA.app/Contents/MacOS` to `/etc/paths`
* Inside the `/Applications/mGBA.app/Contents/MacOS` directory (in a terminal) run: `ln -s mGBA mgba-qt`

# 5. Real hardware - gbafix

In order to be able to play on real hardware or on some emulators, you may need to install 'gbafix'.
The rust implementation can be installed very easily using `cargo install gbafix`.

Make sure that the Cargo bin directory is in your `PATH` as we'll need to use it later.

That is all you need to get started.
You can now move on to 'building the game'.