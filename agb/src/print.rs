use core::fmt::Arguments;

use crate::mgba::{DebugLevel, Mgba};

#[doc(hidden)]
pub fn println(args: Arguments) {
    if let Some(mut mgba) = Mgba::new() {
        let _ = mgba.print(args, DebugLevel::Info);
    }
}

#[doc(hidden)]
pub fn eprintln(args: Arguments) {
    if let Some(mut mgba) = Mgba::new() {
        let _ = mgba.print(args, DebugLevel::Error);
    }
}

/// Works like [`std::println`](https://doc.rust-lang.org/stable/std/macro.println.html).
///
/// Prints to the standard output when running under the mgba emulator.
/// This is mainly useful for debugging, and is reasonably slow.
///
/// ```rust
/// ##![no_std]
/// ##![no_main]
/// # core::include!("doctest_runner.rs");
///
/// # fn test(_: agb::Gba) {
/// agb::println!("Hello, World!");
///
/// let variable = 5;
/// agb::println!("format {variable} argument");
/// # }
/// ```
#[macro_export]
macro_rules! println {
    ($( $x:expr ),*) => {
        $crate::print::println(format_args!($($x,)*))
    };
}

/// Works like [`std::println`](https://doc.rust-lang.org/stable/std/macro.println.html).
///
/// Prints to the standard output when running under the mgba emulator but with the error level internally
/// This is mainly intended for debugging, and is reasonably slow.
///
/// ```rust
/// ##![no_std]
/// ##![no_main]
/// # core::include!("doctest_runner.rs");
///
/// # fn test(_: agb::Gba) {
/// agb::eprintln!("error: Could not load save file");
/// # }
/// ```
#[macro_export]
macro_rules! eprintln {
    ($( $x:expr ),*) => {
        $crate::print::eprintln(format_args!($($x,)*))
    };
}
