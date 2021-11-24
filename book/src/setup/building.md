# Building the game

By the end of this section, you should be able to build and run an example game made using agb!
**This section is optional.**
If you just want to get straight into building your own games, you don't need to do this.
However, we recommended doing this section to prove that your setup works.

# 1. Get the source code

The source code can be fetched using `git clone https://github.com/agbrs/joinedtogether.git`.

# 2. Build the game

Build a copy of the game using `cargo build --release`.
This could take quite a while, but eventually you'll end up with a copy of the game in `target/thumbv4t-none-eabi/release/joinedtogether` or `target/thumbv4t-none-eabi/release/joinedtogether.elf` depending on platform.

This can be run directly by some emulators, but we need to run an extra step in order to convert the elf file into a '.gba' file.

```sh
arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/joinedtogether joinedtogether.gba
gbafix joinedtogether.gba
```

or

```sh
arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/joinedtogether.elf joinedtogether.gba
gbafix joinedtogether.gba
```

And then load the resulting file in your emulator of choice.
That's all there is to it!

If you have `mgba-qt` in your path, then you can launch the game directly using `cargo run --release`