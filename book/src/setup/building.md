# Building the template

By the end of this section, you should be able to build and run the **agb** template.

# 1. Get the source code

The source code can be fetched using `git clone https://github.com/agbrs/template.git`.

# 2. Build the template

Build a copy of the template using `cargo build --release`.
This could take quite a while, but eventually you'll end up with a copy of the template in `target/thumbv4t-none-eabi/release/template` or `target/thumbv4t-none-eabi/release/template.elf` depending on platform.

This can be run directly by some emulators, but we need to run an extra step in order to convert the elf file into a '.gba' file.

```sh
arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/template template.gba
gbafix template.gba
```

or

```sh
arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/template.elf template.gba
gbafix template.gba
```

And then load the resulting file in your emulator of choice.
That's all there is to it!

If you have `mgba-qt` in your path, then you can launch the template directly using `cargo run --release`.