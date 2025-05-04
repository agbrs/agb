#![no_std]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::cast_lossless)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::return_self_not_must_use)]
//! Fixed point number implementation for representing non integers efficiently.
//!
//! If you are using this crate from within `agb`, you should refer to it as `agb::fixnum` rather than `agb_fixnum`.
//! This crate is updated in lockstep with `agb`.

mod num;
mod rect;
mod vec2;

#[doc(hidden)]
pub mod __private {
    pub use const_soft_float;
}

pub use num::*;
pub use rect::*;
pub use vec2::*;
