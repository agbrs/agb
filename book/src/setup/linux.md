# Linux setup

This guide has been tested on Ubuntu, Arch Linux and Raspberry Pi OS running on a raspberry pi 4.

# 1. Install a recent version of rust

To use agb, you'll need to use nightly rust since it requires a few nightly features.
Firstly, ensure that you have **rustup** installed which you can do by following the instructions on the [rust website](https://www.rust-lang.org/tools/install)

If you have already installed rustup, you can update it with `rustup update`.

# 2. arm-none-eabi

To assemble the small amount of assembly in agb and to do the final linking, you'll need to install the `arm-none-eabi` binutils.

* On Debian and derivatives (like Ubuntu): `sudo apt install binutils-arm-none-eabi`
* On Arch Linux and derivatives: `pacman -S arm-none-eabi-binutils`

# 3. git

The source code for the game is hosted on github, so you will need to install git.

* On Debian and derivatives (like Ubuntu): `sudo apt install git`
* On Arch Linux and derivatives: `pacman -S git`

# 4. gbafix

In order to be able to play games made with agb on real hardware or on some emulators, you will need to install 'gbafix'.
The rust implementation can be installed very easily using `cargo install gbafix`.

Make sure that the Cargo bin directory is in your `PATH` as we'll need to use it later.

That is all you need to get started!
You can now move on to 'building the game'.