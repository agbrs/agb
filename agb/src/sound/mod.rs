//! # Game Boy Advance audio
//!
//! The GBA has 2 different ways of producing sound, which agb has support for.
//! You currently cannot use both at the same time, and currently there is no
//! compile time prevention of using both, but you should either use the DMG
//! which allows for Game Boy and Game Boy Color style sound effects, or the mixer
//! which allows for more advanced sounds.
//!
//! The [`dmg`](crate::sound::dmg) module is very rudimentary and doesn't support most of the possible
//! sounds possible. However, it may be expanded in the future.
//!
//! The [`mixer`](crate::sound::mixer) module is high performance, and allows for playing wav files at
//! various levels of quality. Check out the module documentation for more.

pub mod dmg;

pub mod mixer;
