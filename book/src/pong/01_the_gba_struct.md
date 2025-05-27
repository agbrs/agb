# The Gba struct

In this section, we'll cover the importance of the Gba struct and how it gets created for you.

# The importance of the Gba struct

The [`Gba` singleton struct](https://docs.rs/agb/latest/agb/struct.Gba.html) is a crucial part of `agb` game development.
It is used for almost all interactions with the Game Boy Advance's hardware, such as graphics rendering, timer access and audio playback.

You should not create the `Gba` struct yourself.
Instead, it is passed to your main function as an owned reference.
This allows rust's borrow checker to ensure that access to the Game Boy Advance hardware is done in a safe and sensible manner, preventing two bits of your code from modifying data in the wrong way.

# How all agb games start

To use the Gba struct in your `agb` game, you'll need to create a function (normally called `main`).
You should then annotate that function with the [`#[agb::entry]`](https://docs.rs/agb/latest/agb/attr.entry.html) attribute macro provided by the `agb` crate.

Replace the content of the `main` function with the following:

```rust
// infinite loop for now
loop {
    agb::halt();
}
```

This creates an infinite loop and allows you to start building your game.
We use the [`agb::halt`](https://docs.rs/agb/latest/agb/fn.halt.html) to save some battery life on the console while the infinite loop is happening.

# Running your pong game

At this point, your game won't do much except display a black screen. To run your game, use the `cargo run` command as before.

# What we covered

In this section, we covered the importance of the Gba struct in `agb` game development.
By using the Gba struct as a gatekeeper for all hardware interactions, you can ensure that your code is safe and efficient.
You are now ready to learn about sprites and start getting things onto the screen!
