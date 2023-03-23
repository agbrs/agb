# Introduction

**agb** is a powerful and easy-to-use library for writing games for the Game Boy Advance (GBA) in rust.
It provides an abstracted interface to the hardware, allowing you to take full advantage of its capabilities without needing to know the low-level details of its implementation.

## A little bit about agb

`agb` is a library for making games on the Game Boy Advance using the Rust
programming language. The library's main focus is to provide an abstraction
that allows you to develop games which take advantage of the GBA's capabilities
without needing to have extensive knowledge of its low-level implementation.

agb provides the following features:

* Simple build process with minimal dependencies
* Built in importing of sprites, backgrounds, music and sound effects
* High performance audio mixer
* Easy to use sprite and tiled background usage
* A global allocator allowing for use of both `core` and `alloc`

## Why rust?

Rust is an excellent choice of language for developing games on low-level embedded hardware like the GBA.
Its strong type system, memory safety, and performance optimizations make it well-suited for building reliable and efficient code in this context.

Agb leverages rust's unique features by using the type system to model the GBA's hardware.
This approach helps prevent common programming errors and allows developers to quickly build games that function correctly on the GBA platform.

In addition to safety and correctness, rust's performance optimizations are crucial for developing games on the GBA's slow processor.
With a limited amount of time per frame, every optimization counts, and rust's speed and efficiency help ensure that games built with agb run smoothly on the GBA hardware.

# What is in this book?

This book serves as an introduction to agb, showcasing its capabilities and providing guidance on how to use it to build your own GBA games.
It assumes that you have some experience with rust and game development, and provides detailed explanations of the unique challenges of writing games for the GBA.

# Who is this book for?

This book is ideal for anyone interested in writing games for the GBA using rust.
If you're new to either rust or game development, you may want to start with some introductory resources before diving into this book.
This book assumes a basic understanding of rust syntax and semantics, as well as game development concepts.

# Helpful links

* [agb's GitHub](https://github.com/agbrs/agb) is the primary development hub for the library.
* [agb's Discussion Page](https://github.com/agbrs/agb/discussions) is a helpful forum where you can ask for help on using agb or share your projects with the community.
* [agb's crates.io page](https://crates.io/crates/agb) the latest version of the library on crates.io.
* [agb's documentation](https://docs.rs/agb) is a useful reference for the library's API and features.
* [Awesome Game Boy Advance development](https://github.com/gbdev/awesome-gbadev) is a comprehensive resource for GBA development, with links to popular libraries, emulators, and the friendly gbadev Discord server.
* [Example games](https://github.com/agbrs/agb/releases/latest) built using agb can be found in the `examples.zip` file attached to the latest release. Additionally, you can also check out [The Hat Chooses the Wizard](https://lostimmortal.itch.io/the-hat-chooses-the-wizard), a game written using agb as part of the GMTK 2021 game jam.

In addition to these resources, this book provides step-by-step instructions for getting started with agb.