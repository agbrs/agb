# Introduction

**agb** is a library for writing games for the Game Boy Advance (GBA) in rust.
It is intended to make the process of producing games for the Game Boy Advance as easy as possible by giving you access to the hardware in an abstracted format, which allows you to take advantage of all that it has to offer, without needing to know the specifics of how it is implemented.

# What is in this book?

This book is intended as an introduction to what **agb** has to offer, and should set you up nicely to start writing games of your own.
This book will not give a thorough overview of the specifics of the hardware implementation of the GBA unless it is needed as part of an explanation.
An overview of the hardware can be found in chapter 2.

# Who is this book for?

This book is for:
* **People who want to make games for the GBA.** First and foremost, games written using agb cannot run on any other platform except the GBA and emulators. If you don't want to write a game for the GBA, you should probably use a different library.
* **People who have experience in rust.** Unless the rust specific syntax or semantics are important, we will not discuss details here and instead recommend reading the rust book before coming back.
* **People with experience in writing games.** Game programming is hard, and harder still in rust on a GBA. We recommend writing a game for a more user friendly platform before coming back here.

If you fit into all of those categories, welcome!
It is super rewarding being able to play a game you made yourself on a piece of 20+ year old hardware.

# Helpful links

* [agb's GitHub](https://github.com/agbrs/agb) all development happens here
* [agb's Discussion Page](https://github.com/agbrs/agb/discussions) a forum where you can ask for help on the usage of agb
* [agb's crates.io page](https://crates.io/crates/agb)
* [agb's documentation](https://docs.rs/agb) which is useful if you need a quick reference
* [Awesome Game Boy Advance development](https://github.com/gbdev/awesome-gbadev) contains links to popular libraries, emulators and the super friendly gbadev discord
* [Example game](https://lostimmortal.itch.io/the-hat-chooses-the-wizard) written using agb as part of the 2021 GMTK game jam.