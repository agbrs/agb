# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `HashMap::new()` and `HashSet::new()` are now `const`.

### Fixed

- Fixed performance regression in 0.22 when dealing with backgrounds which haven't changed between frames.
- Fixed a memory leak when creating and destroying AffineBackgrounds a few times.

### Improved

- Improved the performance of the mixer by about 10% in cases where you're playing a lot of mono sounds
  at speeds <= 1.5x (common with trackers).
- Improved the performance of tile allocation and deallocation.

## [0.22.1] - 2025/06/06

### Added

- Added `.partial_update_status()` and `.show_if_done()` methods to `InfiniteScrolledMap` which allow for
  conditional rendering based on whether the tiles are fully updated in the background.

### Fixed

- Dynamic sprites will try garbage collecting sprites before failing to allocate.

## [0.22.0] - 2025/05/31

This release comes with a major change to everything in `agb::display`. Mostly built around the new `GraphicsFrame`
used to show both backgrounds and objects to the screen. See the newly expanded [book](https://agbrs.dev/book)
which covers many topics including advanced articles for more exciting graphical effects and completed
[tutorial](https://agbrs.dev/book/pong/01_introduction.html).

You can now also try all the examples [online](https://agbrs.dev/examples) on your browser, including modifying the
code and seeing how that changes the behaviour of the example!

### Added

- `GraphicsFrame` is now how you manage what gets displayed to the screen. Both for backgrounds and objects.
- Lots more examples.
- The crate is 100% documented!
- `#[agb::entry]` now produces code which is compatible with rust 2024.
- Introduced `agb::display::{Rgb15, Rgb}` to represent colours and allow for basic interpolation and conversion.
- You can now pass any constant expression to the `num!` macro.
- Added `round` methods for `Num` and `Vector2D` that rounds towards the nearest integer.
- Added `clamp_point` and corner methods to `Rect`.
- Added `width` and `height` to `TileData` to track the original size of your backgrounds.
- Added an `eprintln!()` macro which prints at error level for mgba.
- More `num_traits` have been implemeted for `Num<I, N>`.
- The template now includes `extern crate alloc;` to avoid any confusion about whether alloc is supported.
- Blending and windows now work with affine backgrounds as well as regular backgrounds
- Added `animation_frame` to `Tag` to allow for more efficient sprite animations.
- You can now write your own doc tests using the `agb::doctest` macro.

### Changed

- The VRamManager is now global, accessed via the `VRAM_MANAGER`.
- You no longer need to pick a tiled mode, instead this is managed by the `GraphicsFrame`.
- Blending, windows and DMA are managed via the `GraphicsFrame`.
- DMA is now entirely safe and is managed by `HBlankDma`.
- The `agb::display` module has been flattened quite significantly.
- `InfiniteScrolledMap` now takes the tile generation function in `set_pos` rather than in the constructor.
- There are getters and setters for almost all properties in `RegularBackground` and `AffineBackground`.
- Affine tile allocation is now much more reliable, rather than it basically being pure luck whether a tile successfully displays.
- Where `agb` used to accept `u16` values for colours, it now expects the new `Rgb15` type to avoid type confusion.
- The mixer now uses `SoundData` rather than `&[u8]` sound effects and music.
- In debug mode, `Num` will now panic if you're multiplying or dividing when it has too much precision set.
- Palette16 is now in the `agb::display` module.
- You get the `graphics` from `gba.graphics` rather than `gba.display.graphics`.
- DynamicTiles are now automatically freed when they are dropped, so you don't need to manually manage that memory any more.
- Font rendering for backgrounds and objects have been unified via the `Layout` struct, and are rendered using the new
  `ObjectFontRenderer` and `BackgroundFontRenderer`. Both fully support colour changes and other arbitrary tags.
- `AffineMatrixInstance` is now `AffineMatrixObject` and the existing `AffineMatrixObject` has been removed to simplify the conversion.
- `AffineMatrix` is now generic on the integer type.
- `sin` and `cos` for `Num<I, N>` has been improved to use a look-up table with linear interpolation rather than a 3rd order polynomial.
- Object's `set_position()` function was renamed to `set_pos()`.
- `include_background_gfx!` now requires a `mod` keyword before the module name.
- `inculde_aseprite!` now creates static variables for each tag rather than using constant string matching.
- Unmanaged objects are now managed by the `GraphicsFrame` rather than `OamIterator` and now only make their changes when `.commit()` is called.
- `ObjectUnmanaged` is now `Object`.
- Affine objects are now a separate type `ObjectAffine`.
- For compatibility with rust 2024, `rng::gen()` has been renamed to `rng::next_u32()`.

### Removed

- Removed bitmap modes. These were accidentally unsound and don't play well with objects or other tiled backgrounds. Some thought needs to go
  into the API to bring these back.
- Removed the dmg audio module. It will return in future but with a better thought out API which will work together with the existing mixer.
- Removed `VRamManager.set_background_palette_raw` since it is actually unsafe and didn't work as intended for smaller palettes in release mode.
- Removed the `syscall` module and put the useful `halt` method in `agb::halt()`.
- Removed the `mgba` module since you should do all printing via the `println!()` macro.
- Removed 'managed' objects. The new `GraphicsFrame` uses a similar paradigm to the old 'unmanaged objects'. But these are now just `Object`.
- Removed `mixer.enable()`. This is now done by default so you just don't need to call it.

## [0.21.3] - 2025/02/01

## [0.21.2] - 2025/02/01

### Added

- Optional serde support for agb-hashmap and agb-fixnum via the `serde` feature flag. The flag is also exposed in the agb crate.
- Added `set_background_palette` to be able to set a single background palette.

### Changed

- Using neither of the testing and backtrace features will mean there is no panic handler.

### Fixed

- Fixed build error due to breaking change in `xmrs`.
- The backtrace feature works on ARM now.

## [0.21.1] - 2024/10/02

### Added

- Added support for s3m and mod format files to `agb-tracker`.
- Added a `HashSet` implementation in `agb::hashmap`.

### Changed

- Changed how 16 colour palettes are optimised to give better average case results. You should find that
  either your palettes will always import, or never import correctly.

## [0.21.0] - 2024/09/24

### Added

- Added support for vibrato in `agb-tracker`'s XM format
- Method to stop the tracker

### Changed

- `agb-tracker` now has an `agb` feature which you must enable to use it with `agb`. You won't notice
  unless you've been using `default-features = false` in which case we recommend removing that from your `Cargo.toml`.

### Fixed

- There are no longer gaps between tiles in affine graphics modes.

### Added

- Added option to export imported background graphics from `include_background_gfx` as pub.

## [0.20.5] - 2024/06/18

### Fixed

- Resolved incompatibility with a dependency's update in `agb-tracker`. If you
  are already using `agb-tracker`, then this won't yet cause an issue as your
  lockfile will maintain the working version. However if start a new project, or
  update dependencies, cargo will choose the later incompatible version.

## [0.20.4] - 2024/06/13

### Changed

- `manhattan_distance` and `magnitude_squared` no longer require the fixed point number.

## [0.20.3] - 2024/06/12

### Added

- Added `find_colour_index_16` and `find_colour_index_256` to the `VRamManager` to find where a colour is in a palette.
- Added `set_graphics_mode` to unmanaged sprites. This allows you to change to the blending and window modes.

### Fixed

- Affine background center position didn't work outside of the upper left quadrant of the gba's screen.
- Fixed backtrace pointing to the wrong line of code (being out by one).
- Fixes overflow caused by certain font characteristics on boundaries of sprites in the object text renderer.

## [0.20.2] - 2024/05/25

### Fixed

- Fixed the crash screen to show debug text even if the qr code fails to generate.
- Fixed the crash screen to prevent the qr code always failing to generate.

## [0.20.1] - 2024/05/17

### Added

- Added `dot` and `cross` product methods for `Vector2D`.

### Fixed

- Fixed an issue with agb tracker where XM files with linear frequencies were playing the wrong notes

## [0.20.0] - 2024/05/14

### Added

- Added a new crash screen which provides a mechanism for seeing a full stack trace of your program when it panics.
  This requires a change to your `.cargo/config.toml`. You must add the rust flag `"-Cforce-frame-pointers=yes"` to
  your rustflags field. This can also be disabled by removing the `backtrace` feature.
- Initial unicode support for font rendering.
- Kerning support for font rendering.
- Added `set_next` method to `OamIterator` to avoid repeated boilerplate when dealing with unmanaged objects.

### Fixed

- Export the `dma` module correctly so you can write the types from it and use it in more complex cases.

### Changed

- Many macros now emit statics rather than consts OR can be used as statics OR
  have had examples changed to use statics. You should use statics where possible
  for assets as consts can lead to them being included multiple times in the
  ROM.
- Fixnums are now implemented with `num_traits` trait definitions.
- Rather than having our own sync with Statics, use the standard portable
  atomics crate. These are reexported for convenience.
- `Mgba` no longer implements `Write`. You're unlikely to notice as
  `agb::println!` is unchanged.
- Writes of long messages to mgba are split over multiple log messages if they
  overflow mgba's buffer. On a panic, only the final message will be Fatal with
  the preceding ones (if needed) being Info.

## [0.19.1] - 2024/03/06

### Added

- `.abs()` on `Vector2D` and `Rect`

### Fixed

- `InfiniteScrolledMap` can now scroll more than 1 tile in a single frame without corrupting.

## [0.19.0] - 2024/03/06

### Added

- Added `.priority()`, `.set_priority()` and `.is_visible()` to `RegularMap`, `AffineMap` and `InfiniteScrolledMap`.
- Replaced `.show()` and `.hide()` with `.set_visible()`in `RegularMap`, `AffineMap` and `InfiniteScrolledMap`.
- Added `.into_inner()` to `InfiniteScrolledMap` to get the map back once you are done using it in the `InfiniteScrolledMap`.
- Added `.hflip()`, `.vflip()`, `.priority()`, `.position()` to `ObjectUnmanaged` and `Object`.
- An abstraction over hblank DMA to allow for cool effects like gradients and circular windows. See the dma_effect\* examples.
- Expermental and incomplete support for MIDI files with agb-tracker.
- Fixnum now implements [`num::Num`](https://docs.rs/num/0.4/num/trait.Num.html) from the [`num`](https://crates.io/crates/num) crate.
- `Default` implementations for `RandomNumberGenerator`, `InitOnce` and `RawMutex`.

### Changed

- A few functions which previously accepted a `Vector<u16>` now accept an `impl Into<Vector2D<u16>>` instead.

## [0.18.1] - 2024/02/06

### Added

- You can now use include_aseprite and include_background_gfx to include files from the out directory using the `$OUT_DIR` token.
- Added `.pause()` and `.resume()` methods to `SoundChannels` to let you pause and resume from where you left off.

## [0.18.0] - 2023/10/31

### Added

- There is now a multiboot feature which you can use to easily make multiboot ROMs.
- Can now set palette on a TileSetting struct.

### Changed

- You no longer need the gba.ld or gba_mb.ld files in your repository. You should delete these when upgrading.

### Fixed

- Multiboot builds now work on mgba.
- Fixed inaccuracy in cosine implementation caused by accidentally multiplying correction term by zero.

## [0.17.1] - 2023/10/05

### Fixed

- Fixed the build on docs.rs.

## [0.17.0] - 2023/10/03

### Added

- New tracker for playing XM files (see the `agb-tracker` crate).
- You can now declare where looping sound channels should restart.
- Fixnums now have constructors from_f32 and from_f64. This is mainly useful if using agb-fixnum outside of the Game Boy Advance e.g. in build scripts or macros.
- New option when loading a background to automatically deduplicate tiles.
- Methods on tile_setting to toggle its hflip and vflip status.

### Changed

- Sound channel panning and volume options are now `Num<i16, 8>` rather than `Num<i16, 4>` for improved precision and sound quality.
- Due to dependency changes, agb-gbafix is now released under MPL rather than GPL.
- `include_background_gfx!` now produces tile sets and tile settings directly.

### Fixed

- 256-colour backgrounds are better supported.
- Mono looping samples will now correctly play to the end if it doesn't perfectly align with a buffer boundry and short samples now also loop correctly.
- Fixed a bug in bitmap4 that caused setting pixels to be always incorrect.

## [0.16.0] - 2023/07/18

### Added

- New `include_palette` macro for including every colour in an image as a `u16` slice.
- New object based text renderer.

### Changed

- Changed the default template game.
- `DynamicSprite` has a new API which changes the constructor and adds a `set_pixel` and `clear` methods.
- You no longer need to install arm-none-eabi-binutils. In order to write games using `agb`, you now only need to install rust nightly.
- 10% performance improvement with the software mixer.

### Fixed

- Compile error if you tried to import a larger sprite which uses more than 15 colours between frames.

## [0.15.0] - 2023/04/25

### Added

- You can now import aseprite files directly (in addition to the already supported png and bmp files) when importing background tiles.
- New additional unmanaged object API for interacting with a more straightforward manner with the underlying hardware.

### Changed

- Importing background tiles has been improved. You no longer need to use `include_gfx!` with the toml file. Instead, use `include_background_gfx`. See the documentation for usage.
- The hashmap implementation is now it its own crate, `agb-hashmap`. There is no change in API, but you can now use this for interop between non-agb code and agb code.
- Moved the existing object API to be the OamManaged API. The old names persist with deprecated notices on them.

## [0.14.0] - 2023/04/11

### Added

- Added custom `gbafix` implementation which can take the elf file produced by `cargo build` directly, removing the need for the objcopy step.

### Changed

- Made Vector2D::new a const function.
- The template now uses rust 2021 edition by default.
- All objects which should only be created once now have the correct lifetimes to only allow one to exist.
- Template now uses codegen-units=1 to workaround bug in nightly.
- Allocator is no longer interrupt safe.
- Soundness issues with interrupts resolved which makes them unsafe and require the closure to be static (breaking change).

### Fixed

- Alpha channel is now considered by `include_gfx!()` even when `transparent_colour` is absent.
- 256 colour backgrounds are now correctly rendered (breaking change).
- The `#[agb::entry]` macro now reports errors better.
- Added the shstrtab section to the linker to ensure that agb builds with lld.

## [0.13.0] - 2023/01/19

### Added

- Added missed implementations of `regular()` and `affine()` to `Tiled1` which made `Tiled1` impossible to use.

### Changed

- Text renderer can now be re-used which is useful for rpg style character/word at a time text boxes.
- Audio now automatically uses interrupts, so you can remove the `setup_interrupt_handler` or `after_vblank` calls to the mixer.
- If a vblank happens outside of `wait_for_vblank`, then next call will immediately return.

### Fixed

- Zero volume now plays no sound.
- Fixed issue where volume was incorrect for volumes which were powers of 2.

## [0.12.2] - 2022/10/22

This is a minor release to fix an alignment issue with background tiles.

### Fixed

- Corrected alignment of background tiles which was causing issues with rendering tiles in some cases.

## [0.12.1] - 2022/10/12

This is a minor release to fix the build of the docs on [docs.rs/agb](https://docs.rs/agb).

### Fixed

- Fixed the agb crate's docs.rs build

## [0.12.0] - 2022/10/11

This version of `agb` has some exciting new features we'd like to highlight and some brand new contributors!

1. Save support for multiple cartridge types (contributed by @Lymia)
2. Affine background support (contributed by @lifning)

We also had a contribution by @ijc8. We can't thank you all enough!

### Added

- Custom allocator support using the `Allocator` trait for `HashMap`. This means the `HashMap` can be used with `InternalAllocator` to allocate to IWRAM or the `ExternalAllocator` to explicitly allocate to EWRAM.
- Support for using windows on the GBA. Windows are used to selectively enable rendering of certain layers or effects.
- Support for the blend mode of the GBA. Blending allows for alpha blending between layers and fading to black and white.
- Added a new agb::sync module that contains GBA-specific synchronization primitives.
- Added support for save files.
- Added implementation of `HashMap.retain()`.
- Added support for affine backgrounds (tiled modes 1 and 2) which allows for scaling, rotating etc of tiled backgrounds.
- Added support for 256 colour backgrounds (when working with affine ones).
- Added affine matrix module. This allows for manipulation of affine matricies for use in backgrounds and in the future objects.
- Added support for dynamic sprites generated at runtime, some parts of this may change significantly so breaking changes are expected here.

### Changed

- Many of the places that originally disabled IRQs now use the `sync` module, reducing the chance of missed interrupts.
- HashMap iterators now implement `size_hint` which should result in slightly better generation of code using those iterators.
- Transparency of backgrounds is now set once in the toml file rather than once for every image.
- Palette generation now takes into account every single background a toml definition rather than one at a time, you can now find it in the PALETTES constant rather than in every individual image.
- Sound frequency is no longer a crate feature, instead set when initialising the sound mixer.
- `testing` is now a default feature, so you no longer need to add a separate `dev-dependencies` line for `agb` in order to enable unit tests for your project.

### Fixed

- Fixed the fast magnitude function in agb_fixnum. This is also used in fast_normalise. Previously only worked for positive (x, y).
- Fixed formatting of fixed point numbers in the range (-1, 0), which previously appeared positive.

## [0.11.1] - 2022/08/02

Version 0.11.1 brings documentation for fixed point numbers. We recommend all users upgrade to this version since it also includes fixes to a few functions in fixnum. See changed section for breaking changes.

### Added

- Support for sprites that are not square.
- Docs for fixed point numbers.

### Changed

- `Rect::contains_point` now considers points on the boundary to be part of the rectangle.
- Signature of `Rect::overlapping_rect` changed to return an Option. Returns None if rectangles don't overlap.

### Fixed

- Fixed point sine calculates the sine correctly.

## [0.10.0] - 2022/07/31

Version 0.10.0 brings about many new features. As with most `agb` upgrades, you will need to update your `gba.ld` and `gba_mb.ld` files which you can find in the [template repo](https://github.com/agbrs/template). We would also recommend copying the `[profile.dev]` and `[profile.release]` sections from `Cargo.toml` if you don't have these values already.

### Added

- [Hyperspace roll](https://lostimmortal.itch.io/hyperspace-roll), a new game built for the GMTK Game Jam 2022 using `agb`. The source code can be found in the `examples` directory.
- Started using GitHub discussions as a forum
- Many functions previously undocumented are now documented
- Z-Ordering of sprites - you can now change the render order of sprites rather than it just being defined by the order in which they appear in object memory
- 32kHz audio. Probably the best sound quality you'll get out of the hardware, but uses a lot of ROM space
- Transparent sprite support with aseprite
- You can now write tests for projects depending on agb
- Very basic font rendering - looking for feedback, this API is far from stable
- Faster implementation of memcpy and memset thanks to the agbabi crate which provide a big performance boost for any project using agb
- If you wish, you can now optionally do dynamic memory allocation to IWRAM instead of only EWRAM
- You can now use 64x64px sprites
- You can now configure the background size for tiled backgrounds
- It is possible to create 'dynamic tiles' for backgrounds. These are tiles which are defined at runtime
- Random number generator in agb::rng

### Changed

- Audio system optimisations - reduced CPU usage by more than 50%
- Background tiles are now removed from Video RAM during `commit()` if they are no longer used rather than immediately reducing flickering
- Improved the README for both the main agb crate and the template
- The template now builds with optimisations in debug mode and debug symbols in release mode
- Added `#[must_use]` to many of the places it is needed
- All subcrates get released at once, so versions are kept in lockstep
- A few methods accepting `Num<..>` have been changed to accept `impl Into<Num<..>>` to make them easier to use

### Removed

- The ability to use timer0 and timer1 through the `timer` module. This was done in order to fully support 32kHz audio

### Fixed

- Sprite data is now correctly aligned so fast copies will always work
- A few methods which should really be internal have had `pub` removed
- The crate now compiles (but does not run) doctests in CI which pointed out a large number of non-compiling examples
