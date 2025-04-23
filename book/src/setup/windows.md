# Windows setup

This guide has been tested on Windows 11 using PowerShell with elevated rights _(don't use cmd)_.

# 1. Install a recent version of rust

To use agb, you'll need to use nightly rust since it requires a few nightly features.
Firstly, ensure that you have **rustup** installed which you can do by following the instructions on the [rust website](https://www.rust-lang.org/tools/install)

If you have installed rustup, you can update it with `rustup update`.  
If the `rustup`-command fails, you'll most probably add the cargo/bin folder to the Path-environment variable.

# 2. git

The source code for the game is hosted on github, so you will need to install git.

You'd need to follow this official github git [guide](https://github.com/git-guides/install-git).

# 3. mGBA

We recommend using the mGBA emulator which you can download from [here](https://mgba.io/downloads.html).

After installing, you can add the binary to your Path-environment variable and create an alias for the agb run command to use.

Creating link for mgba-qt:

```PS
New-Item -itemtype hardlink -path "C:\Program Files\mGBA\mgba-qt.exe" -value "C:\Program Files\mGBA\mGBA.exe"
```

# 4. gbafix

In order to be able to play games made with agb on real hardware or on some emulators, you will need to install 'agb-gbafix'.
Agb's implementation can be installed very easily using `cargo install agb-gbafix`.

That is all you need to get started!
You can now move on to 'building the game'.
