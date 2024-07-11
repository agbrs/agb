#![no_std]
// This appears to be needed for testing to work
#![cfg_attr(any(test, feature = "testing"), no_main)]
#![cfg_attr(any(test, feature = "testing"), feature(custom_test_frameworks))]
#![cfg_attr(
    any(test, feature = "testing"),
    test_runner(crate::test_runner::test_runner)
)]
#![cfg_attr(
    any(test, feature = "testing"),
    reexport_test_harness_main = "test_main"
)]
#![feature(allocator_api)]
#![feature(asm_const)]
#![warn(clippy::all)]
#![allow(clippy::needless_pass_by_ref_mut)]
#![deny(clippy::must_use_candidate)]
#![deny(clippy::trivially_copy_pass_by_ref)]
#![deny(clippy::semicolon_if_nothing_returned)]
#![deny(clippy::map_unwrap_or)]
#![deny(clippy::needless_pass_by_value)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![deny(clippy::cloned_instead_of_copied)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::invalid_html_tags)]

//! # agb
//! `agb` is a library for making games on the Game Boy Advance using the Rust
//! programming language. It attempts to be a high level abstraction over the
//! internal workings of the Game Boy Advance whilst still being high
//! performance and memory efficient.
//!
//! To get started with agb, you should clone the [template repo](https://github.com/agbrs/template) and work from there.

/// This macro is used to convert a png, bmp or aseprite file into a format usable by the Game Boy Advance.
///
/// Suppose you have a file in `examples/water_tiles.png` which contains some tiles you'd like to use.
///
/// You import them using:
/// ```rust,no_run
/// ##![no_std]
/// ##![no_main]
/// agb::include_background_gfx!(water_tiles, tiles => "examples/water_tiles.png");
/// ```
///
/// This will generate something along the lines of the following:
///
/// ```rust,ignore
/// // module name comes from the first argument, name of the constant from the arrow
/// mod water_tiles {
///     pub static tiles = /* ... */;
/// }
/// ```
///
/// And tiles will be an instance of [`TileData`][crate::display::tile_data::TileData]
///
/// You can import multiple files at once, and the palette data will be combined so they can all be visible.
///
/// # Examples
///
/// Assume the tiles are loaded as above
///
/// In `src/main.rs`:
/// ```rust,no_run
/// ##![no_std]
/// ##![no_main]
/// #
/// use agb::{
///     display::{
///         tiled::{RegularBackgroundSize, TileFormat, TileSet, TileSetting, Tiled0, TiledMap, VRamManager},
///         Priority,
///     },
///     include_background_gfx,
/// };
///
/// agb::include_background_gfx!(water_tiles, tiles => "examples/water_tiles.png");
///
/// # fn load_tileset(mut gfx: Tiled0, mut vram: VRamManager) {
/// let tileset = &water_tiles::tiles.tiles;
///
/// vram.set_background_palettes(water_tiles::PALETTES);
///
/// let mut bg = gfx.background(Priority::P0, RegularBackgroundSize::Background32x32, tileset.format());
///
/// for y in 0..20u16 {
///     for x in 0..30u16 {
///         bg.set_tile(
///             &mut vram,
///             (x, y),
///             tileset,
///             water_tiles::tiles.tile_settings[0],
///         );
///     }
/// }
/// bg.commit(&mut vram);
/// bg.set_visible(true);
/// # }
/// ```
///
/// Including from the out directory is supported through the `$OUT_DIR` token.
///
/// ```rust,ignore
/// # #![no_std]
/// # #![no_main]
/// # use agb::include_background_gfx;
/// include_background_gfx!(generated_background, "000000", DATA => "$OUT_DIR/generated_background.aseprite");
/// ```
pub use agb_image_converter::include_background_gfx;

#[doc(hidden)]
pub use agb_image_converter::include_aseprite_inner;

#[doc(hidden)]
pub use agb_image_converter::include_font as include_font_inner;

#[doc(hidden)]
pub use agb_image_converter::include_colours_inner;

#[macro_export]
macro_rules! include_font {
    ($font_path: literal, $font_size: literal) => {{
        use $crate::display;
        $crate::include_font_inner!($font_path, $font_size)
    }};
}

/// This macro declares the entry point to your game written using `agb`.
///
/// It is already included in the template, but your `main` function must be annotated with `#[agb::entry]`, takes 1 argument and never returns.
/// Doing this will ensure that `agb` can correctly set up the environment to call your rust function on start up.
///
/// # Examples
/// ```no_run,rust
/// #![no_std]
/// #![no_main]
///
/// use agb::Gba;
///
/// #[agb::entry]
/// fn main(mut gba: Gba) -> ! {
///     loop {}
/// }
/// ```
pub use agb_macros::entry;

pub use agb_sound_converter::include_wav;

extern crate alloc;
mod agb_alloc;

mod agbabi;
#[cfg(feature = "backtrace")]
mod backtrace;
mod bitarray;
/// Implements everything relating to things that are displayed on screen.
pub mod display;
/// Provides access to the GBA's direct memory access (DMA) which is used for advanced effects
pub mod dma;
/// Button inputs to the system.
pub mod input;
/// Interacting with the GBA interrupts
pub mod interrupt;
mod memory_mapped;
/// Implements logging to the mgba emulator.
pub mod mgba;
#[doc(inline)]
pub use agb_fixnum as fixnum;
/// Contains an implementation of a hashmap which suits the gameboy advance's hardware.
pub use agb_hashmap as hash_map;
#[cfg(feature = "backtrace")]
mod panics_render;
/// Simple random number generator
pub mod rng;
pub mod save;
mod single;
/// Implements sound output.
pub mod sound;
/// A module containing functions and utilities useful for synchronizing state.
mod sync;
/// System BIOS calls / syscalls.
pub mod syscall;
/// Interactions with the internal timers
pub mod timer;

mod no_game;

/// Default game
pub use no_game::no_game;

pub(crate) mod arena;
mod global_asm;

pub mod external {
    pub use critical_section;
    pub use once_cell;
    pub use portable_atomic;
}

pub use {agb_alloc::ExternalAllocator, agb_alloc::InternalAllocator};

#[cfg(not(any(test, feature = "testing")))]
#[panic_handler]
#[allow(unused_must_use)]
fn panic_implementation(info: &core::panic::PanicInfo) -> ! {
    avoid_double_panic(info);

    if let Some(mut mgba) = mgba::Mgba::new() {
        let _ = mgba.print(format_args!("{info}"), mgba::DebugLevel::Fatal);
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

// If we panic during the panic handler, then there isn't much we can do any more. So this code
// just infinite loops halting the CPU.
fn avoid_double_panic(info: &core::panic::PanicInfo) {
    static IS_PANICKING: portable_atomic::AtomicBool = portable_atomic::AtomicBool::new(false);

    if IS_PANICKING.load(portable_atomic::Ordering::SeqCst) {
        if let Some(mut mgba) = mgba::Mgba::new() {
            let _ = mgba.print(
                format_args!("Double panic: {info}"),
                mgba::DebugLevel::Fatal,
            );
        }
        loop {
            syscall::halt();
        }
    } else {
        IS_PANICKING.store(true, portable_atomic::Ordering::SeqCst);
    }
}

/// The Gba struct is used to control access to the Game Boy Advance's hardware in a way which makes it the
/// borrow checker's responsibility to ensure no clashes of global resources.
///
/// This is will be created for you via the [`#[agb::entry]`][entry] attribute.
///
/// # Examples
///
/// ```no_run,rust
/// #![no_std]
/// #![no_main]
///
/// use agb::Gba;
///
/// #[agb::entry]
/// fn main(mut gba: Gba) -> ! {
///     // Do whatever you need to do with gba
///
///     loop {}
/// }
/// ```
#[non_exhaustive]
pub struct Gba {
    /// Manages access to the Game Boy Advance's display hardware
    pub display: display::Display,
    /// Manages access to the Game Boy Advance's beeps and boops sound hardware as part of the
    /// original Game Boy's sound chip (the DMG).
    pub sound: sound::dmg::Sound,
    /// Manages access to the Game Boy Advance's direct sound mixer for playing raw wav files.
    pub mixer: sound::mixer::MixerController,
    /// Manages access to the Game Boy Advance cartridge's save chip.
    pub save: save::SaveManager,
    /// Manages access to the Game Boy Advance's 4 timers.
    pub timers: timer::TimerController,
    /// Manages access to the Game Boy Advance's DMA
    pub dma: dma::DmaController,
}

impl Gba {
    #[doc(hidden)]
    #[must_use]
    /// # Safety
    ///
    /// May only be called a single time. It is not needed to call this due to
    /// it being called internally by the [`entry`] macro.
    pub unsafe fn new_in_entry() -> Self {
        Self::single_new()
    }

    const unsafe fn single_new() -> Self {
        Self {
            display: display::Display::new(),
            sound: sound::dmg::Sound::new(),
            mixer: sound::mixer::MixerController::new(),
            save: save::SaveManager::new(),
            timers: timer::TimerController::new(),
            dma: dma::DmaController::new(),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
/// *Unstable* support for running tests using `agb`
///
/// In order to use this, you need to enable the unstable `custom_test_framework` feature and copy-paste
/// the following into the top of your application:
///
/// ```rust,ignore
/// #![cfg_attr(test, feature(custom_test_frameworks))]
/// #![cfg_attr(test, reexport_test_harness_main = "test_main")]
/// #![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
/// ```
///
/// With this support, you will be able to write tests which you can run using `mgba-test-runner`.
/// Tests are written using `#[test_case]` rather than `#[test]`.
///
/// ```rust,ignore
/// #[test_case]
/// fn test_ping_pong(_gba: &mut Gba) {
///     assert_eq!(1, 1);
/// }
/// ```
///
/// You can run the tests using `cargo test`, but it will work better through `mgba-test-runner` by
/// running something along the lines of `CARGO_TARGET_THUMBV4T_NONE_EABI_RUNNER=mgba-test-runner cargo test`.
pub mod test_runner {
    use super::*;

    #[doc(hidden)]
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
            mgba::test_runner_measure_cycles();
            self(gba);
            mgba::test_runner_measure_cycles();

            mgba.print(format_args!("[ok]"), mgba::DebugLevel::Info)
                .unwrap();
        }
    }

    #[panic_handler]
    fn panic_implementation(info: &core::panic::PanicInfo) -> ! {
        avoid_double_panic(info);

        #[cfg(feature = "backtrace")]
        let frames = backtrace::unwind_exception();

        if let Some(mut mgba) = mgba::Mgba::new() {
            let _ = mgba.print(format_args!("[failed]"), mgba::DebugLevel::Error);
        }

        #[cfg(feature = "backtrace")]
        crate::panics_render::render_backtrace(&frames, info);

        #[cfg(not(feature = "backtrace"))]
        loop {
            syscall::halt();
        }
    }

    static mut TEST_GBA: Option<Gba> = None;

    #[doc(hidden)]
    pub fn test_runner(tests: &[&dyn Testable]) {
        let mut mgba = mgba::Mgba::new().unwrap();
        mgba.print(
            format_args!("Running {} tests", tests.len()),
            mgba::DebugLevel::Info,
        )
        .unwrap();

        let gba = unsafe { TEST_GBA.as_mut() }.unwrap();

        for test in tests {
            test.run(gba);
        }

        mgba.print(
            format_args!("Tests finished successfully"),
            mgba::DebugLevel::Info,
        )
        .unwrap();
    }

    // needed to fudge the #[entry] below
    #[cfg(test)]
    mod agb {
        pub mod test_runner {
            pub use super::super::agb_start_tests;
        }
    }

    #[cfg(test)]
    #[entry]
    fn agb_test_main(gba: Gba) -> ! {
        #[allow(clippy::empty_loop)]
        loop {} // full implementation provided by the #[entry]
    }

    #[doc(hidden)]
    pub fn agb_start_tests(gba: Gba, test_main: impl Fn()) -> ! {
        unsafe { TEST_GBA = Some(gba) };
        test_main();
        #[allow(clippy::empty_loop)]
        loop {}
    }

    pub fn assert_image_output(image: &str) {
        display::busy_wait_for_vblank();
        display::busy_wait_for_vblank();
        let mut mgba = crate::mgba::Mgba::new().unwrap();
        mgba.print(format_args!("image:{image}"), crate::mgba::DebugLevel::Info)
            .unwrap();
        display::busy_wait_for_vblank();
    }
}

#[inline(never)]
pub(crate) fn program_counter_before_interrupt() -> u32 {
    extern "C" {
        static mut agb_rs__program_counter: u32;
    }
    unsafe { agb_rs__program_counter }
}

#[cfg(test)]
mod test {
    use core::ptr::addr_of_mut;

    use super::Gba;

    #[test_case]
    #[allow(clippy::eq_op)]
    fn trivial_test(_gba: &mut Gba) {
        assert_eq!(1, 1);
    }

    #[test_case]
    fn gba_struct_is_zero_sized(_gba: &mut Gba) {
        use core::mem;
        assert_eq!(mem::size_of::<Gba>(), 0);
    }

    #[test_case]
    fn wait_30_frames(_gba: &mut Gba) {
        let vblank = crate::interrupt::VBlank::get();
        let mut counter = 0;
        loop {
            if counter > 30 {
                break;
            }
            vblank.wait_for_vblank();
            counter += 1;
        }
    }

    #[link_section = ".ewram"]
    static mut EWRAM_TEST: u32 = 5;
    #[test_case]
    fn ewram_static_test(_gba: &mut Gba) {
        unsafe {
            let ewram_ptr = addr_of_mut!(EWRAM_TEST);
            let content = ewram_ptr.read_volatile();
            assert_eq!(content, 5, "expected data in ewram to be 5");
            ewram_ptr.write_volatile(content + 1);
            let content = ewram_ptr.read_volatile();
            assert_eq!(content, 6, "expected data to have increased by one");
            let address = ewram_ptr as usize;
            assert!(
                (0x0200_0000..0x0204_0000).contains(&address),
                "ewram is located between 0x0200_0000 and 0x0204_0000, address was actually found to be {address:#010X}",
            );
        }
    }

    #[link_section = ".iwram"]
    static mut IWRAM_EXPLICIT: u32 = 9;
    #[test_case]
    fn iwram_explicit_test(_gba: &mut Gba) {
        unsafe {
            let iwram_ptr = addr_of_mut!(IWRAM_EXPLICIT);
            let address = iwram_ptr as usize;
            assert!(
                (0x0300_0000..0x0300_8000).contains(&address),
                "iwram is located between 0x0300_0000 and 0x0300_8000, but was actually found to be at {address:#010X}"
            );
            let c = iwram_ptr.read_volatile();
            assert_eq!(c, 9, "expected content to be 9");
            iwram_ptr.write_volatile(u32::MAX);
            let c = iwram_ptr.read_volatile();
            assert_eq!(c, u32::MAX, "expected content to be {}", u32::MAX);
        }
    }

    static mut IMPLICIT_STORAGE: u32 = 9;
    #[test_case]
    fn implicit_data_test(_gba: &mut Gba) {
        unsafe {
            let iwram_ptr = addr_of_mut!(IMPLICIT_STORAGE);
            let address = iwram_ptr as usize;
            assert!(
                (0x0200_0000..0x0204_0000).contains(&address),
                "implicit data storage is expected to be in ewram, which is between 0x0300_0000 and 0x0300_8000, but was actually found to be at {address:#010X}"
            );
            let c = iwram_ptr.read_volatile();
            assert_eq!(c, 9, "expected content to be 9");
            iwram_ptr.write_volatile(u32::MAX);
            let c = iwram_ptr.read_volatile();
            assert_eq!(c, u32::MAX, "expected content to be {}", u32::MAX);
        }
    }
}
