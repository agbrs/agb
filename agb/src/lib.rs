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
//! `agb` is a library for making games on the Game Boy Advance using rust.
//!
//! The library's main focus is to provide an abstraction that allows you to develop games which take advantage of the GBA's
//! capabilities without needing to have extensive knowledge of its low-level implementation.
//!
//! `agb` provides the following features:
//! * Simple build process with minimal dependencies
//! * Built in importing of sprites, backgrounds, music and sound effects
//! * High performance audio mixer
//! * Easy to use sprite and tiled background usage
//! * A global allocator allowing for use of both core and alloc
//!
//! A more detailed walkthrough can be found in [the book](https://agbrs.dev/book), or you can play with the
//! [interactive examples](https://agbrs.dev/examples) to get a better feel of what's possible.

/// Include background tiles from a png, bmp or aseprite file.
///
/// This macro is used to convert a png, bmp or aseprite file into a format usable by the Game Boy Advance.
///
/// Suppose you have a file in `examples/gfx/beach-background.aseprite` which contains some tiles you'd like to use.
///
/// You import them using:
/// ```rust,no_run
/// ##![no_std]
/// ##![no_main]
/// agb::include_background_gfx!(
///     mod backgrounds,
///     BEACH => "examples/gfx/beach-background.aseprite"
/// );
/// ```
///
/// This will generate something along the lines of the following:
///
/// ```rust,ignore
/// // module name comes from the first argument, name of the constant from the arrow
/// mod backgrounds {
///     pub static BEACH: TileData = /* ... */;
///     pub static PALETTES: Palette16[] = /* ... */;
/// }
/// ```
///
/// And `BEACH` will be an instance of [`TileData`][crate::display::tile_data::TileData]
///
/// You can import multiple files at once, and the palette data will be combined so they can all be visible.
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// agb::include_background_gfx!(
///     mod backgrounds,
///     BEACH => "examples/gfx/beach-background.aseprite",
///     HUD => "examples/gfx/hud.aseprite",
/// );
/// ```
///
/// # Palettes
///
/// The Game Boy Advance, in 16-colour mode can have at most 16 palettes each of size 16.
/// Each tile can only refer to a single one of those palettes.
/// `include_background_gfx!` will try its best to arrange the colours in the palettes
/// such that the passed background file can be displayed.
///
/// However, this isn't always possible if your background has too many colours in one tile,
/// or too many varieties of palettes between the individual tiles.
/// If this happens, then the call to `include_background_gfx!` will fail at compile time.
/// You can fix this by either importing as 256 colours, or by changing your backgrounds to
/// use fewer colour variations.
///
/// # Transparent backgrounds
///
/// The GBA supports a single transparent colour. Any pixels marked with full alpha transparency
/// in the background will be mapped to the first colour of the relevant palette, which is displayed
/// as transparent.
///
/// However, that transparency colour will be the one shown behind any background so any space which
/// has no tiles, or you can see all the way through will be shown using that colour.
///
/// You can configure which colour that will be with the optional second argument to `include_background_gfx!`
///
/// ```rust,no_run
/// ##![no_std]
/// ##![no_main]
/// agb::include_background_gfx!(
///     mod backgrounds,
///     "00bdfe", // the sky colour hex code
///     BEACH => "examples/gfx/beach-background.aseprite",
///     HUD => "examples/gfx/hud.aseprite",
/// );
/// ```
///
/// # Deduplication
///
/// If your background has a large number of repeated 8x8 tiles (like the beach background above),
/// then you can let the tile importing do the hard bit of deduplicating those tiles and with that
/// you'll save some video RAM, which will then allow you to use even more tiles.
///
/// Note that once you've used deduplication, you need to use the [`TileData::settings`](display::display_data::TileData.settings)
/// field in order to be able to actually display your given tiles. This is because the tiles
/// could be flipped horizontally or vertically (or both) and combined with other tiles.
///
/// ```rust,no_run
/// ##![no_std]
/// ##![no_main]
/// agb::include_background_gfx!(
///     mod backgrounds,
///     BEACH => deduplicate "examples/gfx/beach-background.aseprite",
/// );
/// ```
///
/// # 256 colours
///
/// The Game Boy Advance supports both 16-colour and 256-colour tiles. If you're using 256 colours
/// in some (or all of) your backgrounds, you'll have to include them in 256 colour mode. You are
/// required to use 256 colour backgrounds with affine tiles.
///
/// ```rust,no_run
/// ##![no_std]
/// ##![no_main]
/// agb::include_background_gfx!(
///     mod backgrounds,
///     BEACH => 256 "examples/gfx/beach-background.aseprite",
///     HUD => "examples/gfx/hud.aseprite", // you can still import 16-colour backgrounds at the same time
/// );
/// ```
///
/// # Module visibility
///
/// The resulting module that's being exported can have a different visibility if you want it to.
/// So for instance you could make the resulting module `pub` or `pub(crate)` as follows:
///
/// ```rust,no_run
/// ##![no_std]
/// ##![no_main]
/// agb::include_background_gfx!(
///     pub mod backgrounds,
///     BEACH => "examples/gfx/beach-background.aseprite",
/// );
///
/// agb::include_background_gfx!(
///     pub(crate) mod backgrounds2,
///     BEACH => "examples/gfx/beach-background.aseprite",
/// );
/// ```
///
/// # `$OUT_DIR`
///
/// You may be generating the backgrounds as part of your `build.rs` file. If you're doing that, you'll
/// want to put the generated files in `$OUT_DIR`. You can refer to this as part of the file name:
///
/// ```rust,ignore
/// # #![no_std]
/// # #![no_main]
/// # use agb::include_background_gfx;
/// include_background_gfx!(mod generated_background, "000000", DATA => "$OUT_DIR/generated_background.aseprite");
/// ```
///
/// # Examples
///
/// ## `fill_with` and displaying a full screen background
///
/// This example uses [`RegularBackgroundTiles::fill_with`](display::tiled::RegularBackgroundTiles::fill_with)
/// to fill the screen with a screen-sized image.
///
/// ```rust
/// ##![no_std]
/// ##![no_main]
/// # core::include!("doctest_runner.rs");
/// use agb::{
///     display::{
///         tiled::{RegularBackgroundSize, TileFormat, TileSet, TileSetting, RegularBackgroundTiles, VRAM_MANAGER},
///         Priority,
///     },
///     include_background_gfx,
/// };
///
/// agb::include_background_gfx!(
///     pub mod backgrounds,
///     BEACH => "examples/gfx/beach-background.aseprite",
/// );
///
/// # fn test(_: agb::Gba) {
/// VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);
///
/// let mut bg = RegularBackgroundTiles::new(
///     Priority::P0,
///     RegularBackgroundSize::Background32x32,
///     TileFormat::FourBpp,
/// );
/// bg.fill_with(&backgrounds::BEACH);
/// # }
/// ```
///
/// ## Combining modifiers
///
/// Modifiers can be combined, so you can import and deduplicate a 256 colour background.
///
/// ```rust,no_run
/// ##![no_std]
/// ##![no_main]
/// agb::include_background_gfx!(
///     mod backgrounds,
///     BEACH => 256 deduplicate "examples/gfx/beach-background.aseprite",
///     HUD => deduplicate "examples/gfx/hud.aseprite", // you can still import 16-colour backgrounds at the same time
/// );
/// ```
pub use agb_image_converter::include_background_gfx;

#[doc(hidden)]
pub use agb_image_converter::include_aseprite_inner;

#[doc(hidden)]
pub use agb_image_converter::include_font as include_font_inner;

#[doc(hidden)]
pub use agb_image_converter::include_colours_inner;

#[doc(hidden)]
pub use agb_image_converter::include_aseprite_256_inner;

#[macro_export]
/// Includes a ttf font to be usable by dynamic font rendering.
///
/// The first parameter is the filepath and the second is the point size of the font.
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// # core::include!("doctest_runner.rs");
/// use agb::{display::font::Font, include_font};
///
/// static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);
/// # fn test(gba: agb::Gba) {}
/// ```
macro_rules! include_font {
    ($font_path: literal, $font_size: literal) => {{
        use $crate::display::font::{Font, FontLetter};
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

#[doc(hidden)]
pub use agb_sound_converter::include_wav as include_wav_inner;

/// Include a wav file to be used for sound effects or music.
///
/// The parameter is the path to the sound file relative to the root of your crate.
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// # core::include!("doctest_runner.rs");
/// use agb::{sound::mixer::SoundData, include_wav};
///
/// static JUMP_SOUND: SoundData = include_wav!("examples/sfx/jump.wav");
/// # fn test(gba: agb::Gba) {}
/// ```
#[macro_export]
macro_rules! include_wav {
    ($filepath: literal) => {{
        use $crate::sound::mixer::SoundData;
        $crate::include_wav_inner!($filepath)
    }};
}

extern crate alloc;
mod agb_alloc;

mod agbabi;
#[cfg(feature = "backtrace")]
mod backtrace;
/// Implements everything relating to things that are displayed on screen.
pub mod display;
/// Provides access to the GBA's direct memory access (DMA) for advanced graphical effects.
pub mod dma;
/// Button inputs to the system.
pub mod input;
/// Interacting with the GBA interrupts.
pub mod interrupt;
mod memory_mapped;
/// Implements logging to the mgba emulator.
pub(crate) mod mgba;
#[doc(inline)]
pub use agb_fixnum as fixnum;
/// Contains an implementation of a hashmap which suits the Game Boy Advance's hardware.
pub use agb_hashmap as hash_map;
#[cfg(feature = "backtrace")]
mod panics_render;
#[doc(hidden)]
pub mod print;
pub(crate) mod refcount;
/// Simple random number generator.
pub mod rng;
pub mod save;
mod single;
/// Implements sound output.
pub mod sound;
/// A module containing functions and utilities useful for synchronizing state.
mod sync;
/// System BIOS calls / syscalls.
pub(crate) mod syscall;
/// Interactions with the internal timers.
pub mod timer;
pub(crate) mod util;

mod no_game;
pub use no_game::no_game;

mod global_asm;

/// Re-exports of situationally useful crates for GBA development
///
/// `agb` will refer to these types directly, so if you need anything from
/// any of the referred to crates, you can use these references to avoid needing
/// to match version numbers in your game vs. the `agb` crate's version.
pub mod external {
    pub use critical_section;
    pub use once_cell;
    pub use portable_atomic;
}

pub use {agb_alloc::ExternalAllocator, agb_alloc::InternalAllocator};

#[cfg(any(test, feature = "testing", feature = "backtrace"))]
#[panic_handler]
fn panic_implementation(info: &core::panic::PanicInfo) -> ! {
    avoid_double_panic(info);

    #[cfg(feature = "backtrace")]
    let frames = backtrace::unwind_exception();

    #[cfg(feature = "testing")]
    if let Some(mut mgba) = mgba::Mgba::new() {
        let _ = mgba.print(format_args!("[failed]"), mgba::DebugLevel::Error);
    }

    #[cfg(feature = "backtrace")]
    crate::panics_render::render_backtrace(&frames, info);

    #[cfg(not(feature = "backtrace"))]
    if let Some(mut mgba) = mgba::Mgba::new() {
        let _ = mgba.print(format_args!("{info}"), mgba::DebugLevel::Fatal);
    }

    #[cfg(not(feature = "backtrace"))]
    loop {
        halt();
    }
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
            halt();
        }
    } else {
        IS_PANICKING.store(true, portable_atomic::Ordering::SeqCst);
    }
}

/// Controls access to the Game Boy Advance's hardware.
///
/// This struct exists to make it the borrow checker's responsibility to ensure no clashes of global resources.
/// It will be created for you via the [`#[agb::entry]`][entry] attribute.
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
///     loop {
///         agb::halt();
///     }
/// }
/// ```
#[non_exhaustive]
pub struct Gba {
    /// Manages access to the Game Boy Advance's display hardware
    pub graphics: display::GraphicsDist,
    /// Manages access to the Game Boy Advance's direct sound mixer for playing raw wav files.
    pub mixer: sound::mixer::MixerController,
    /// Manages access to the Game Boy Advance cartridge's save chip.
    pub save: save::SaveManager,
    /// Manages access to the Game Boy Advance's 4 timers.
    pub timers: timer::TimerController,
}

impl Gba {
    #[doc(hidden)]
    #[must_use]
    /// # Safety
    ///
    /// May only be called a single time. It is not needed to call this due to
    /// it being called internally by the [`entry`] macro.
    pub unsafe fn new_in_entry() -> Self {
        unsafe {
            display::object::SPRITE_LOADER.init();
            display::tiled::VRAM_MANAGER.initialise();

            Self::single_new()
        }
    }

    const unsafe fn single_new() -> Self {
        Self {
            graphics: display::GraphicsDist,
            mixer: sound::mixer::MixerController::new(),
            save: save::SaveManager::new(),
            timers: timer::TimerController::new(),
        }
    }
}

/// Halts the CPU until an interrupt occurs.
///
/// The CPU is switched to a low-power mode but all other subsystems continue running.
/// You would mainly use this if you are stopping the game, and want to put an infinite loop without
/// using 100% of the CPU.
///
/// Once an interrupt occurs, this function will return.
///
/// ```rust,no_run
/// #![no_std]
/// #![no_main]
///
/// use agb::Gba;
///
/// #[agb::entry]
/// fn main(mut gba: Gba) -> ! {
///     // your game code here    
///
///     loop {
///         agb::halt();
///     }
/// }
/// ```
pub fn halt() {
    syscall::halt();
}

#[cfg(any(test, feature = "testing"))]
/// *Unstable* support for running tests using `agb`.
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
/// fn dummy_test(_gba: &mut Gba) {
///     assert_eq!(1, 1);
/// }
/// ```
///
/// You can run the tests using `cargo test`, but it will work better through `mgba-test-runner` by
/// running something along the lines of `CARGO_TARGET_THUMBV4T_NONE_EABI_RUNNER=mgba-test-runner cargo test`.
pub mod test_runner {
    use util::SyncUnsafeCell;

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

    static TEST_GBA: SyncUnsafeCell<Option<Gba>> = SyncUnsafeCell::new(None);

    #[doc(hidden)]
    pub fn test_runner(tests: &[&dyn Testable]) {
        let mut mgba = mgba::Mgba::new().unwrap();
        mgba.print(
            format_args!("Running {} tests", tests.len()),
            mgba::DebugLevel::Info,
        )
        .unwrap();

        let gba = unsafe { &mut *TEST_GBA.get() }.as_mut().unwrap();

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
    fn agb_test_main(_gba: Gba) -> ! {
        #[allow(clippy::empty_loop)]
        loop {} // full implementation provided by the #[entry]
    }

    #[doc(hidden)]
    pub fn agb_start_tests(gba: Gba, test_main: impl Fn()) -> ! {
        *unsafe { &mut *TEST_GBA.get() } = Some(gba);
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
    unsafe extern "C" {
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

    #[unsafe(link_section = ".ewram")]
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

    #[unsafe(link_section = ".iwram")]
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
