//! Implements everything relating to things which are displayed on the screen.
//!
//! Games written using `agb` typically follow the ['update-render loop'](https://gameprogrammingpatterns.com/game-loop.html).
//! The way your components update will be very dependent on the game you are writing, but each frame you would normally do the following:
//!
//! ```rust
//! # #![no_std]
//! # #![no_main]
//! # core::include!("../doctest_runner.rs");
//! use agb::display::GraphicsFrame;
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
//! Further sections e.g. [`Blend`], [`Windows`] and [`dma`](crate::dma) will go into more detail about other effects you can apply once
//! you've mastered the content of this article.
//!
//! ## `.show(frame: &mut GraphicsFrame)`
//!
//! The most common pattern involving [`GraphicsFrame`] you'll see in the `agb` library is a `.show()` method which typically
//! accepts a mutable reference to a [`GraphicsFrame`] e.g. [`RegularBackgroundTiles::show`](tiled::RegularBackgroundTiles::show) and
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
#![warn(missing_docs)]
use crate::{interrupt::VBlank, memory_mapped::MemoryMapped};

use alloc::boxed::Box;
use bilge::prelude::*;

use tiled::{BackgroundFrame, DisplayControlRegister, TiledBackground};

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

pub mod affine;
mod blend;
mod window;

pub mod font;

const DISPLAY_CONTROL: MemoryMapped<DisplayControlRegister> =
    unsafe { MemoryMapped::new(0x0400_0000) };
pub(crate) const DISPLAY_STATUS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0004) };
const VCOUNT: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0006) };

pub use blend::{
    Blend, BlendAlphaEffect, BlendFadeEffect, BlendLayer, BlendObjectTransparency, Layer,
};

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
        Graphics::new(Oam::new(), unsafe { TiledBackground::new() }, VBlank::get())
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
/// # core::include!("../doctest_runner.rs");
/// # use agb::Gba;
/// # fn test(mut gba: Gba) {
/// use agb::display::{
///     Priority,
///     tiled::{RegularBackgroundTiles, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
/// };
///
/// // This is an instance of Graphics
/// let mut gfx = gba.graphics.get();
///
/// let bg = RegularBackgroundTiles::new(
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
    tiled: TiledBackground<'gba>,
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
    fn new(oam: Oam<'gba>, tiled: TiledBackground<'gba>, vblank: VBlank) -> Self {
        Self {
            oam,
            tiled,
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
            bg_frame: self.tiled.iter(),
            blend: Blend::new(),
            windows: Windows::new(),
            next_dma: None,
            others: &mut self.others,
        }
    }
}

/// Manages everything to do with the current frame that is being rendered.
///
/// Any effects you want to apply to this frame are done between the call to
/// [`gfx.frame()`](Graphics::frame) and [`frame.commit()`](GraphicsFrame::commit).
///
/// Normally you'll want to pass the current `&mut frame` to a `.show()` method
/// for example [`RegularBackgroundTiles::show`](tiled::RegularBackgroundTiles::show)
/// or [`Object::show`](object::Object::show).
pub struct GraphicsFrame<'frame> {
    pub(crate) oam_frame: OamFrame<'frame>,
    pub(crate) bg_frame: BackgroundFrame<'frame>,
    blend: Blend,
    windows: Windows,
    next_dma: Option<Box<dyn DmaFrame>>,

    others: &'frame mut Others,
}

impl GraphicsFrame<'_> {
    /// Commit the next frame to the screen.
    ///
    /// This will first wait for the current frame to finish rendering before going ahead
    /// and doing all the steps required to display the next frame on the screen.
    pub fn commit(mut self) {
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
