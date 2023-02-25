# Learn agb part I: Pong

In this section, you'll learn how to make a simple pong-style game for the Game Boy Advance using agb.
By following the steps in this section below, you'll gain an understanding of:

* How to use tiled graphics modes.
* How to import graphics using `agb`.
* What Game Boy Advance sprites are, how to create them, and how to display them on the screen.
* How to detect button input and use it to control game objects.
* How to add a static background to your game.
* How to make a dynamic background to display scores.
* How to add music and sound effects to your game.

With this knowledge, you'll be well equipped to start making your own games for the GBA!

## Getting started

To get started, create a new repository based on the [agb template](https://github.com/agbrs/template) and name it `pong`.

Next, update the `name` field in `Cargo.toml` to `pong` like so:

```toml
[package]
name = "pong"
version = "0.1.0"
authors = ["Your name here"]
edition = "2021"

# ...
```

Now, you're ready to dive and and start learning about `agb`!