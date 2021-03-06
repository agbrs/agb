#![no_std]
#![feature(asm)]
#![deny(clippy::all)]

use core::fmt::Write;
pub mod display;
pub mod input;
pub mod interrupt;

mod memory_mapped;
mod mgba;
mod single;

pub mod syscall;

#[panic_handler]
#[allow(unused_must_use)]
fn panic_implementation(info: &core::panic::PanicInfo) -> ! {
    if let Some(mut mgba) = mgba::Mgba::new() {
        write!(mgba, "{}", info);
        mgba.set_level(mgba::DebugLevel::Fatal);
    }

    loop {}
}

static mut GBASINGLE: single::Singleton<Gba> = single::Singleton::new(unsafe { Gba::single_new() });

pub struct Gba {
    pub display: display::Display,
}

impl Gba {
    pub fn new() -> Self {
        unsafe { GBASINGLE.take() }
    }

    const unsafe fn single_new() -> Self {
        Self {
            display: display::Display::new(),
        }
    }
}

impl Default for Gba {
    fn default() -> Self {
        Self::new()
    }
}
