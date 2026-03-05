//! Implements everything relating to things which are displayed on the screen.
//!
//! Games written using `agb` typically follow the ['update-render loop'](https://gameprogrammingpatterns.com/game-loop.html).
//! The way your components update will be very dependent on the game you are writing, but each frame you would normally do the following:
//!
//! ```rust
//! # #![no_std]
//! # #![no_main]
//! use agb::display::GraphicsFrame;
//! # #[agb::doctest]
//! # fn test(mut gba: agb::Gba) {
//!
//! let mut my_game = MyGame::new();
//! let mut gfx = gba.graphics.get();
//!
//! loop {
//!     my_game.update();
//!
//!     let mut frame = gfx.frame();
//!     my_game.show(&mut frame);
//!     frame.commit();
//!     # break
//! }
//! # }
//! # struct MyGame { }
//! # impl MyGame {
//! #     fn new() -> Self { Self {} }
//! #
//! #     fn update(&mut self) {
//! #         // update the game state
//! #     }
//! #
//! #     fn show(&self, frame: &mut GraphicsFrame) {
//! #         // do all the showing of things on screen
//! #     }
//! # }
//! ```
//!
//! The [`GraphicsFrame`] is the key mechanism for displaying anything on the screen (the `frame` variable you see above).
//! Further sections e.g. [`Blend`], [`Windows`] and [`dma`] will go into more detail about other effects you can apply once
//! you've mastered the content of this article.
//!
//! ## `.show(frame: &mut GraphicsFrame)`
//!
//! The most common pattern involving [`GraphicsFrame`] you'll see in the `agb` library is a `.show()` method which typically
//! accepts a mutable reference to a [`GraphicsFrame`] e.g. [`RegularBackground::show`](tiled::RegularBackground::show) and
//! [`Object::show`](object::Object::show).
//!
//! Due to this naming convention, it is also conventional in games written using `agb` to name the `render` method `show()`
//! and have the same method signature.
//! You should not be doing any mutation of state during the `show()` method, and as much loading and other CPU intensive
//! work as possible should be done prior to the call to `show()`.
//!
//! See the [frame lifecycle](https://agbrs.dev/examples/frame_lifecycle) example for a simple walkthrough for how to
//! manage a frame with a single player character.
//!
//! ## `.commit()`
//!
//! Once everything you want to be visible on the frame is ready, you should follow this up with a call to `.commit()` on the frame.
//! This will wait for the current frame to finish rendering before quickly setting everything up for the next frame.
//!
//! This method takes ownership of the current `frame` instance, so you won't be able to use it for any further calls once this is done.
//! You will need to create a new frame object from the `gfx` instance.
use crate::{dma, interrupt::VBlank, memory_mapped::MemoryMapped};

use alloc::{borrow::Cow, boxed::Box};
use bilge::prelude::*;

use tiled::{BackgroundFrame, DisplayControlRegister, VRAM_MANAGER};

use object::{Oam, OamFrame, initilise_oam};

pub use colours::{Rgb, Rgb15, include_colours};
pub use palette16::Palette16;

/// Graphics mode 3. Bitmap mode that provides a 16-bit colour framebuffer.
pub(crate) mod bitmap3;
mod colours;
pub mod object;
/// Palette type.
mod palette16;
/// Data produced by agb-image-converter
pub mod tile_data;
pub mod tiled;
pub mod utils;

mod affine;
pub use affine::AffineMatrix;
mod blend;
mod window;

pub mod font;

const DISPLAY_CONTROL: MemoryMapped<DisplayControlRegister> =
    unsafe { MemoryMapped::new(0x0400_0000) };
pub(crate) const DISPLAY_STATUS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0004) };
const VCOUNT: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0006) };

pub use blend::{Blend, BlendAlphaEffect, BlendFadeEffect, BlendObjectTransparency, Layer};

pub use window::{MovableWindow, WinIn, Window, Windows};

/// Width of the Game Boy advance screen in pixels
pub const WIDTH: i32 = 240;
/// Height of the Game Boy advance screen in pixels
pub const HEIGHT: i32 = 160;

/// Use to get the [`Graphics`] subsystem for `agb`.
///
/// You'll find this as part of the [`Gba`](crate::Gba) struct.
#[non_exhaustive]
pub struct GraphicsDist;

impl GraphicsDist {
    /// Get the [`Graphics`] from the [`Gba`](crate::Gba) struct.
    pub fn get(&mut self) -> Graphics<'_> {
        unsafe { initilise_oam() };
        Graphics::new(Oam::new(), VBlank::get())
    }
}

/// Manage the graphics for the Game Boy Advance.
///
/// Handles objects and backgrounds. The main thing you'll want from this struct is the
/// [`GraphicsFrame`] returned by the [`frame()`](Graphics::frame) method.
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// # use agb::Gba;
/// # #[agb::doctest]
/// # fn test(mut gba: Gba) {
/// use agb::display::{
///     Priority,
///     tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
/// };
///
/// // This is an instance of Graphics
/// let mut gfx = gba.graphics.get();
///
/// let bg = RegularBackground::new(
///     Priority::P0,
///     RegularBackgroundSize::Background32x32,
///     TileFormat::FourBpp
/// );
///
/// // load the background with some tiles
///
/// loop {
///     let mut frame = gfx.frame();
///     bg.show(&mut frame);
///     frame.commit();
///     # break;
/// }
/// # }
/// ```
pub struct Graphics<'gba> {
    oam: Oam<'gba>,
    others: Others,
}

pub(crate) trait DmaFrame {
    fn commit(&mut self);
    fn cleanup(&mut self);
}

struct Others {
    vblank: VBlank,
    dma: Option<Box<dyn DmaFrame>>,
}

impl<'gba> Graphics<'gba> {
    fn new(oam: Oam<'gba>, vblank: VBlank) -> Self {
        Self {
            oam,
            others: Others { vblank, dma: None },
        }
    }

    /// Start a new frame.
    ///
    /// See the [display module level documentation](crate::display) for details on how to use
    /// the graphics frame.
    pub fn frame(&mut self) -> GraphicsFrame<'_> {
        GraphicsFrame {
            oam_frame: self.oam.frame(),
            bg_frame: BackgroundFrame::default(),
            blend: Blend::new(),
            windows: Windows::new(),
            next_dma: None,
            others: &mut self.others,

            background_palette: Cow::Borrowed(VRAM_MANAGER.vram_background_palette()),
        }
    }
}

/// Manages everything to do with the current frame that is being rendered.
///
/// Any effects you want to apply to this frame are done between the call to
/// [`gfx.frame()`](Graphics::frame) and [`frame.commit()`](GraphicsFrame::commit).
///
/// Normally you'll want to pass the current `&mut frame` to a `.show()` method
/// for example [`RegularBackground::show`](tiled::RegularBackground::show)
/// or [`Object::show`](object::Object::show).
pub struct GraphicsFrame<'frame> {
    pub(crate) oam_frame: OamFrame<'frame>,
    pub(crate) bg_frame: BackgroundFrame,
    blend: Blend,
    windows: Windows,
    next_dma: Option<Box<dyn DmaFrame>>,

    others: &'frame mut Others,

    background_palette: Cow<'static, [Palette16]>,
}

impl GraphicsFrame<'_> {
    /// Commit the next frame to the screen.
    ///
    /// This will first wait for the current frame to finish rendering before going ahead
    /// and doing all the steps required to display the next frame on the screen.
    pub fn commit(mut self) {
        // In embassy mode, VBlank waiting is handled by embassy-agb
        #[cfg(not(feature = "embassy"))]
        self.others.vblank.wait_for_vblank();

        core::mem::swap(&mut self.others.dma, &mut self.next_dma);

        if let Some(mut old) = self.next_dma.take() {
            old.cleanup();
        }

        self.oam_frame.commit();
        self.bg_frame.commit();
        self.blend.commit();
        self.windows.commit();

        if let Some(dma) = self.others.dma.as_mut() {
            dma.commit();
        }

        if let Cow::Owned(background) = self.background_palette {
            VRAM_MANAGER.set_background_palettes(&background);
        }

        // the bg_frame for this frame is still valid, so the GC won't remove anything that
        // is actually still visible, but will remove as much as possible to leave room for
        // the next frame's graphics.
        VRAM_MANAGER.gc();
    }

    /// Sets the `pal_index` background palette to the 4bpp one given in `palette`.
    /// Note that `pal_index` must be in the range 0..=15 as there are only 16 palettes available on
    /// the GameBoy Advance.
    pub fn set_background_palette(&mut self, pal_index: u8, palette: &Palette16) {
        self.background_palette.to_mut()[pal_index as usize] = palette.clone();
    }

    /// Sets all background palettes based on the entries given in `palettes`. Note that the GameBoy Advance
    /// can have at most 16 palettes loaded at once, so only the first 16 will be loaded (although this
    /// array can be shorter if you don't need all 16).
    ///
    /// Unlike [`VRAM_MANAGER.set_background_palettes()`](crate::display::tiled::VRamManager::set_background_palettes) which
    /// takes effect immediately, this version takes effect on [`.commit()`](GraphicsFrame::commit).
    /// This ensures that palette changes are synchronised with background and other graphical updates.
    pub fn set_background_palettes(&mut self, palettes: &[Palette16]) {
        self.background_palette.to_mut()[..palettes.len()].clone_from_slice(palettes);
    }

    /// Used if you want to control a colour in the background which could change e.g. on every row of pixels.
    /// Very useful if you want a gradient of more colours than the gba can normally handle.
    ///
    /// See [`HBlankDma`](crate::dma::HBlankDma) for examples for how to do this, or the
    /// [`dma_effect_background_colour`](https://agbrs.dev/examples/dma_effect_background_colour) example.
    #[must_use]
    pub fn background_palette_colour_dma(
        &self,
        pal_index: usize,
        colour_index: usize,
    ) -> dma::DmaControllable<Rgb15> {
        VRAM_MANAGER.background_palette_colour_dma(pal_index, colour_index)
    }

    /// Used if you want to control a colour in the background which could change e.g. on every row of pixels.
    /// Very useful if you want a gradient of more colours than the gba can normally handle.
    ///
    /// See [`HBlankDma`](crate::dma::HBlankDma) for examples for how to do this, or the
    /// [`dma_effect_background_colour`](https://agbrs.dev/examples/dma_effect_background_colour) example.
    #[must_use]
    pub fn background_palette_colour_256_dma(
        &self,
        colour_index: usize,
    ) -> dma::DmaControllable<Rgb15> {
        assert!(colour_index < 256);

        self.background_palette_colour_dma(colour_index / 16, colour_index % 16)
    }

    /// Sets a single colour for a given background palette. Takes effect on commit
    pub fn set_background_palette_colour(
        &mut self,
        pal_index: usize,
        colour_index: usize,
        colour: Rgb15,
    ) {
        self.background_palette.to_mut()[pal_index].update_colour(colour_index, colour);
    }

    /// Sets a single colour in a 256 colour palette. `colour_index` must be less than 256.
    pub fn set_background_palette_colour_256(&mut self, colour_index: usize, colour: Rgb15) {
        assert!(colour_index < 256);
        self.set_background_palette_colour(colour_index / 16, colour_index % 16, colour);
    }

    /// Gets the index of the colour for a given background palette, or None if it doesn't exist
    #[must_use]
    pub fn find_colour_index_16(&self, palette_index: usize, colour: Rgb15) -> Option<usize> {
        self.background_palette[palette_index]
            .colours
            .iter()
            .position(|&c| c == colour)
    }

    /// Gets the index of the colour in the entire background palette, or None if it doesn't exist
    #[must_use]
    pub fn find_colour_index_256(&self, colour: Rgb15) -> Option<usize> {
        for i in 0..16 {
            if let Some(index) = self.find_colour_index_16(i, colour) {
                return Some(i * 16 + index);
            }
        }

        None
    }

    /// Control the blending for this frame.
    pub fn blend(&mut self) -> &mut Blend {
        &mut self.blend
    }

    /// Control the windows for this frame.
    pub fn windows(&mut self) -> &mut Windows {
        &mut self.windows
    }

    pub(crate) fn add_dma<C: DmaFrame + 'static>(&mut self, c: C) {
        self.next_dma = Some(Box::new(c));
    }
}

/// Waits until vblank using a busy wait loop, this should almost never be used.
/// I only say almost because whilst I don't believe there to be a reason to use
/// this I can't rule it out.
pub fn busy_wait_for_vblank() {
    while VCOUNT.get() >= 160 {}
    while VCOUNT.get() < 160 {}
}

/// The priority of a background layer or object. A higher priority should be
/// thought of as rendering first, and so is behind that of a lower priority.
/// For an equal priority background layer and object, the background has a
/// higher priority and therefore is behind the object.
#[bitsize(2)]
#[allow(missing_docs)]
#[derive(FromBits, PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum Priority {
    #[default]
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
}
