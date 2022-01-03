# The Gba struct

In this section, we'll cover the importance of the Gba struct and how to create it.

# The importance of the Gba struct

Almost all interaction with the Game Boy Advance's hardware goes through the [Gba singleton struct](https://docs.rs/agb/latest/agb/struct.Gba.html).
Your games using `agb` will typically create this in the `main` function and then handle the abstractions in there.

The Gba struct is used to take advantage of rust's borrow checker, and lean on it to ensure that access to the Game Boy Advance hardware is done 'sensibly'.
You won't have to worry about 2 bits of your code modifying data in the wrong way!
This struct is a 'singleton', so you cannot create 2 instances of it at once.
Attempting to do so will result in a panic which by default crashes the game.

# How all agb games start

Replace the content of the `main` function with the following:

```rust,ignore
# #![no_std]
# #![no_main]
# extern crate agb;
# #[agb::entry]
# fn main() -> ! {
let mut gba = agb::Gba::new();

loop {} // infinite loop for now
# }
```

and ignore warnings for now.

# Running your pong game

Although there isn't much to see at the moment (just a black screen), you can start the game by using `cargo run` or whatever worked for you in the introduction.

# What we did

This was a very simple but incredibly important part of any game using `agb`.
All interactions with the hardware are gated via the Gba struct, so it must be created at the start of your `main` function and never again.

You are now ready to learn about display modes and how to start getting things onto the screen!