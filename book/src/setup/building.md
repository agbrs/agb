# Building and running the agb template

In this section, you will learn how to build and run the agb template.
By the end of this section, you will have a working GBA game that you can run on your emulator of choice.

## 1. Clone the repository

The first step is to clone the agb template repository using Git.
Open a terminal or command prompt and run the following command:

```sh
git clone https://github.com/agbrs/template.git
```

This will create a copy of the agb template repository on your local machine.

## 2. Build the template

Next, navigate to the `template` directory in the repository and build the template using the following command:

```sh
cd template
cargo build --release
```

This command will compile the agb template in release mode.
The resulting binary file can be found in the `target/thumbv4t-none-eabi/release` directory.
Depending on your platform, the file will have either a `.elf` extension or no extension.

## 3. Convert the binary to a GBA file

In order to run the game on an emulator, we need to convert the binary file to a GBA file.
To do this, we'll use the tool `agb-gbafix`.

Run the following command to convert the binary file to a GBA ROM:

```sh
agb-gbafix target/thumbv4t-none-eabi/release/agb_template -o agb_template.gba
```

or

```sh
agb-gbafix target/thumbv4t-none-eabi/release/agb_template.elf -o agb_template.gba
```

This command will add the correct GBA header to the template.gba file and it will be playable on real hardware or an emulator.

## 4. Run the game

Finally, you can run the game on your emulator of choice.
Load the template.gba file in your emulator, and you should see the agb template running.

If you have mgba-qt installed on your machine, you can run the game directly from the command line using the following command:

```sh
cargo run --release
```

This will build and run the agb template in a single step.

That's it! You now have a working agb template that you can use as a starting point for your own GBA game.
