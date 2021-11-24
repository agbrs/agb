# Linux setup

This guide has been tested on Ubuntu, Arch Linux and Raspberry Pi OS running on a raspberry pi 4.

# 1. Install a recent version of rust

agb unfortunately relies on a few nightly rust features, so you need to ensure you have that installed.
Firstly, ensure that you have **rustup** installed which you can do by following the instructions on the [rust website](https://www.rust-lang.org/tools/install)

You can update rustup with `rustup update` if you have already installed it.

# 2. arm-none-eabi

We need this installed in order to be able to assemble the small amount of assembly in agb, and to do the final linking.

* On Debian and derivatives (like Ubuntu): `sudo apt install binutils-arm-non-eabi`
* On Arch Linux and derivatives: `pacman -S arm-none-eabi-binutils`

# 3. git

The source code for the game is hosted on github, so you will need git installed.

* On Debian and derivatives (like Ubuntu): `sudo apt install git`
* On Arch Linux and derivatives: `pacman -S git`

# 4. gbafix

In order to be able to play on real hardware or on some emulators, you may need to install 'gbafix'.
The rust implementation can be installed very easily using `cargo install gbafix`.

Make sure that the Cargo bin directory is in your `PATH` as we'll need to use it later.

That is all you need to get started.
You can now move on to 'building the game'.