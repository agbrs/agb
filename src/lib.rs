#![no_std]
// This appears to be needed for testing to work
#![cfg_attr(test, no_main)]
#![feature(asm)]
#![deny(clippy::all)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod display;
pub mod input;

mod interrupt;
mod memory_mapped;
pub mod mgba;
mod single;

pub mod syscall;

#[cfg(not(test))]
use core::fmt::Write;

#[cfg(not(test))]
#[panic_handler]
#[allow(unused_must_use)]
fn panic_implementation(info: &core::panic::PanicInfo) -> ! {
    if let Some(mut mgba) = mgba::Mgba::new() {
        write!(mgba, "{}", info);
        mgba.set_level(mgba::DebugLevel::Fatal);
    }

    loop {}
}

#[cfg(not(test))]
static mut GBASINGLE: single::Singleton<Gba> = single::Singleton::new(unsafe { Gba::single_new() });

#[cfg(test)]
static mut GBASINGLE: single::Singleton<Gba> = single::Singleton::empty();

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

pub trait Testable {
    fn run(&self, gba: &mut Gba);
}

impl<T> Testable for T
where
    T: Fn(&mut Gba),
{
    fn run(&self, gba: &mut Gba) {
        let mut mgba = mgba::Mgba::new().unwrap();
        mgba.print(
            format_args!("{}...", core::any::type_name::<T>()),
            mgba::DebugLevel::Info,
        )
        .unwrap();
        self(gba);
        mgba.print(format_args!("[ok]"), mgba::DebugLevel::Info)
            .unwrap();
    }
}

#[panic_handler]
#[cfg(test)]
fn panic_implementation(info: &core::panic::PanicInfo) -> ! {
    if let Some(mut mgba) = mgba::Mgba::new() {
        mgba.print(format_args!("[failed]"), mgba::DebugLevel::Error)
            .unwrap();
        mgba.print(format_args!("Error: {}", info), mgba::DebugLevel::Fatal)
            .unwrap();
    }

    loop {}
}

pub fn test_runner(tests: &[&dyn Testable]) {
    let mut mgba = mgba::Mgba::new().unwrap();
    mgba.print(
        format_args!("Running {} tests", tests.len()),
        mgba::DebugLevel::Info,
    )
    .unwrap();

    let mut gba = unsafe { Gba::single_new() };

    for test in tests {
        test.run(&mut gba);
    }

    mgba.print(
        format_args!("Tests finished successfully"),
        mgba::DebugLevel::Info,
    )
    .unwrap();
}

#[no_mangle]
#[cfg(test)]
pub extern "C" fn main() -> ! {
    test_main();
    loop {}
}

#[test_case]
fn trivial_test(_gba: &mut Gba) {
    assert_eq!(1, 1);
}

#[test_case]
fn wait_30_frames(gba: &mut Gba) {
    let vblank = gba.display.vblank.get();
    let mut counter = 0;
    loop {
        if counter > 30 {
            break;
        }
        vblank.wait_for_VBlank();
        counter += 1
    }
}
