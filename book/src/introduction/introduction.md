# Introduction

**agb** is a powerful and easy-to-use library for writing games for the Game Boy Advance (GBA) in rust.
It provides an abstracted interface to the hardware, allowing you to take full advantage of its capabilities without needing to know the low-level details of its implementation.

`agb` provides the following features:

- Simple build process with minimal dependencies
- Built-in importing of sprites, backgrounds, music and sound effects
- High performance audio mixer
- Easy to use sprite and tiled background usage
- A global allocator allowing for use of both `core` and `alloc`

# Why rust?

Rust is an excellent choice of language for developing games on low-level embedded hardware like the GBA.
Its strong type system and memory safety are incredibly useful when working with platforms without operating system checks, while the zero cost abstractions and performance optimisations allow you to write expressive code which still performs well.

`agb` uses rust's features by using the type system to model the GBA's hardware.
This approach helps prevent common programming errors and allows you to quickly build games that function correctly and make the limitations of the platform clear.

# What is in this book?

This book serves as an introduction to `agb`, showcasing its capabilities and providing guidance on how to use it to build your own GBA games.
It assumes that you have some experience with rust and game development, and provides tutorials to teach you the basics, along with longer articles diving deeper into specific features.

# Who is this book for?

This book is for anyone interested in writing games for the GBA using rust.
If you're new to either rust or game development, you may want to start with some introductory resources before diving into this book.
This book assumes a basic understanding of rust syntax and semantics, as well as game development concepts.

# Helpful links

- [agb's GitHub](https://github.com/agbrs/agb) is the primary development hub for the library.
- [agb's Discussion Page](https://github.com/agbrs/agb/discussions) is a helpful forum where you can ask for help on using agb or share your projects with the community.
- [agb's crates.io page](https://crates.io/crates/agb) the latest version of the library on crates.io.
- [agb's documentation](https://docs.rs/agb) is a useful reference for the library's API and features.
- [Awesome Game Boy Advance development](https://github.com/gbdev/awesome-gbadev) is a comprehensive resource for GBA development, with links to popular libraries, emulators, and the friendly gbadev Discord server.
- [Example games](https://github.com/agbrs/agb/releases/latest) built using agb can be found in the `examples.zip` file attached to the latest release.
  Additionally, you can also check out [this collection](https://itch.io/c/4302342/games-made-with-agb) of games on itch.io.
