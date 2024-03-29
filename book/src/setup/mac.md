# Mac setup

This guide has been tested on MacOS 13.0.1 on an M1 chip.

# 1. Install a recent version of rust

To use agb, you'll need to use nightly rust since it requires a few nightly features.
Firstly, ensure that you have **rustup** installed which you can do by following the instructions on the [rust website](https://www.rust-lang.org/tools/install)

If you have already installed rustup, you can update it with `rustup update`.

# 2. Get git

The source code for the game is hosted on github, so you will need git installed. Follow the instructions at [git-scm.com](https://git-scm.com/)

# 3. GBA Emulator - mGBA

We recommend using the mGBA emulator which you can download for Mac [here](https://mgba.io/downloads.html).

After installing to your `/Applications` folder you can add the binary to your path and create an alias for the agb run command to use.

* Add `/Applications/mGBA.app/Contents/MacOS` to `/etc/paths`
* Inside the `/Applications/mGBA.app/Contents/MacOS` directory (in a terminal) run: `ln -s mGBA mgba-qt`

# 4. Real hardware - gbafix

In order to be able to play games made with agb on real hardware or on some emulators, you will need to install 'agb-gbafix'.
Agb's implementation can be installed very easily using `cargo install agb-gbafix`.

Make sure that the Cargo bin directory is in your `PATH` as we'll need to use it later.

That is all you need to get started!
You can now move on to 'building the game'.