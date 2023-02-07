# Learn agb part I - pong

In this section, we'll make a simple pong style game for the Game Boy Advance using `agb`.
You will learn:

* How to use tiled graphics modes.
* How to import graphics using `agb`.
* What Game Boy Advance sprites are and how to put them on the screen.
* How to detect button input and react to it.
* How to add a static background.
* How to make a dynamic background for a score display.
* How to add music to your game.
* How to add sound effects to your game.

With this knowledge, you'll be well equipped to start making your own games!

## Getting started

To start, create a new repository based on the [agb template](https://github.com/agbrs/template).
We'll call this `pong`.

Then replace the `name` field in `Cargo.toml` with `pong`, to end up with something similar to:

```toml
[package]
name = "pong"
version = "0.1.0"
authors = ["Your name here"]
edition = "2021"

# ...
```

You are now ready to get started learning about how `agb` works.