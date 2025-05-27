# Building and running the agb template

In this section, you will learn how to build and run the [`agb` template](https://github.com/agbrs/template).
By the end of this section, you will have a working GBA game that you can run on your emulator of choice.

# 1. Clone the repository

The first step is to clone the agb template repository using Git.
Open a terminal or command prompt and run the following command:

```sh
git clone https://github.com/agbrs/template.git
```

This will create a copy of the agb template repository on your local machine.

# 2. Build the template

Next, navigate to the `template` directory in the repository and build the template using the following command:

```sh
cd template
cargo build --release
```

This command will compile the agb template in release mode.
The resulting binary file can be found in the `target/thumbv4t-none-eabi/release` directory.
Depending on your platform, the file will have either a `.elf` extension or no extension.

This command will add the correct GBA header to the template.gba file and it will be playable on real hardware or an emulator.

# 3. Run the game

If you have mgba-qt installed on your machine, you can run the game directly from the command line using the following command:

```sh
cargo run --release
```

This will build and run the agb template in a single step.

# 4. Convert the binary to a GBA file

In order to build the game for releasing it, you will need to create a GBA file.
To do this, we'll use the tool `agb-gbafix`.

Run the following command to convert the binary file to a GBA ROM:

```sh
agb-gbafix target/thumbv4t-none-eabi/release/agb_template -o agb_template.gba
```

or

```sh
agb-gbafix target/thumbv4t-none-eabi/release/agb_template.elf -o agb_template.gba
```

You can use this GBA file in an emulator or on real hardware
