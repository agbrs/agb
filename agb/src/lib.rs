#![no_std]
// This appears to be needed for testing to work
#![cfg_attr(test, no_main)]
#![feature(asm)]
#![deny(clippy::all)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

//! # agb
//! `agb` is a library for making games on the Game Boy Advance using the Rust
//! programming language. It attempts to be a high level abstraction over the
//! internal workings of the Game Boy Advance whilst still being high
//! performance and memory efficient.

/// Implements everything relating to things that are displayed on screen.
pub mod display;
/// Button inputs to the system.
pub mod input;
/// Implements sound output.
pub mod sound;

pub use agb_image_converter::include_gfx;

mod bitarray;
mod interrupt;
mod memory_mapped;
/// Implements logging to the mgba emulator.
pub mod mgba;
pub mod number;
mod single;

/// System BIOS calls / syscalls.
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

static mut GBASINGLE: single::Singleton<Gba> = single::Singleton::new(unsafe { Gba::single_new() });

pub struct Gba {
    pub display: display::Display,
    pub sound: sound::dmg::Sound,
    pub mixer: sound::mixer::MixerController,
}

impl Gba {
    pub fn new() -> Self {
        unsafe { GBASINGLE.take() }
    }

    const unsafe fn single_new() -> Self {
        Self {
            display: display::Display::new(),
            sound: sound::dmg::Sound::new(),
            mixer: sound::mixer::MixerController::new(),
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
        mgba::number_of_cycles_tagged(785);
        self(gba);
        mgba::number_of_cycles_tagged(785);
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

    let mut gba = Gba::new();

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

#[cfg(test)]
fn assert_image_output(image: &str) {
    display::busy_wait_for_VBlank();
    display::busy_wait_for_VBlank();
    let mut mgba = crate::mgba::Mgba::new().unwrap();
    mgba.print(
        format_args!("image:{}", image),
        crate::mgba::DebugLevel::Info,
    )
    .unwrap();
    display::busy_wait_for_VBlank();
}

#[cfg(test)]
mod test {
    use super::Gba;

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

    #[link_section = ".ewram"]
    static mut EWRAM_TEST: u32 = 5;
    #[test_case]
    fn ewram_static_test(_gba: &mut Gba) {
        unsafe {
            let ewram_ptr = &mut EWRAM_TEST as *mut u32;
            let content = ewram_ptr.read_volatile();
            assert_eq!(content, 5, "expected data in ewram to be 5");
            ewram_ptr.write_volatile(content + 1);
            let content = ewram_ptr.read_volatile();
            assert_eq!(content, 6, "expected data to have increased by one");
            let address = ewram_ptr as usize;
            assert!(
                address >= 0x0200_0000 && address < 0x0204_0000,
                "ewram is located between 0x0200_0000 and 0x0204_0000, address was actually found to be {:#010X}",
                address
            );
        }
    }

    #[link_section = ".iwram"]
    static mut IWRAM_EXPLICIT: u32 = 9;
    #[test_case]
    fn iwram_explicit_test(_gba: &mut Gba) {
        unsafe {
            let iwram_ptr = &mut IWRAM_EXPLICIT as *mut u32;
            let address = iwram_ptr as usize;
            assert!(
                address >= 0x0300_0000 && address < 0x0300_8000,
                "iwram is located beween 0x0300_0000 and 0x0300_8000, but was actually found to be at {:#010X}",
                address
            );
            let c = iwram_ptr.read_volatile();
            assert_eq!(c, 9, "exctected content to be 9");
            iwram_ptr.write_volatile(u32::MAX);
            let c = iwram_ptr.read_volatile();
            assert_eq!(c, u32::MAX, "expected content to be {}", u32::MAX);
        }
    }

    static mut IMPLICIT_STORAGE: u32 = 9;
    #[test_case]
    fn implicit_data_test(_gba: &mut Gba) {
        unsafe {
            let iwram_ptr = &mut IMPLICIT_STORAGE as *mut u32;
            let address = iwram_ptr as usize;
            assert!(
                address >= 0x0200_0000 && address < 0x0204_0000,
                "implicit data storage is expected to be in ewram, which is between 0x0300_0000 and 0x0300_8000, but was actually found to be at {:#010X}",
                address
            );
            let c = iwram_ptr.read_volatile();
            assert_eq!(c, 9, "exctected content to be 9");
            iwram_ptr.write_volatile(u32::MAX);
            let c = iwram_ptr.read_volatile();
            assert_eq!(c, u32::MAX, "expected content to be {}", u32::MAX);
        }
    }
}
