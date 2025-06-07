#![warn(missing_docs)]
use agb_fixnum::FixedWidthSignedInteger;
use alloc::rc::Rc;
use bilge::prelude::*;
use core::alloc::Layout;

use crate::{
    display::{
        GraphicsFrame, Priority,
        affine::AffineMatrix,
        tiled::{TileFormat, tiles::Tiles},
    },
    fixnum::{Num, Vector2D},
};

use super::{
    AffineBackgroundCommitData, AffineBackgroundData, AffineBackgroundId,
    BackgroundControlRegister, SCREENBLOCK_SIZE, TRANSPARENT_TILE_INDEX, TileIndex, TileSet,
    VRAM_MANAGER,
};

mod screenblock;

pub(crate) use screenblock::AffineBackgroundScreenBlock;

/// The size of the affine background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum AffineBackgroundSize {
    /// 16x16 tiles or 128x128px
    Background16x16 = 0,
    /// 32x32 tiles or 256x256px
    Background32x32 = 1,
    /// 64x64 tiles or 512x512px
    Background64x64 = 2,
    /// 128x128 tiles or 1024x1024px
    Background128x128 = 3,
}

/// Whether the background should wrap at the edges.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum AffineBackgroundWrapBehaviour {
    /// Don't wrap and instead show the default background colour.
    NoWrap = 0,
    /// Wrap at the edges similar to the behaviour for regular backgrounds.
    Wrap = 1,
}

impl AffineBackgroundSize {
    fn width(self) -> usize {
        match self {
            AffineBackgroundSize::Background16x16 => 16,
            AffineBackgroundSize::Background32x32 => 32,
            AffineBackgroundSize::Background64x64 => 64,
            AffineBackgroundSize::Background128x128 => 128,
        }
    }

    fn num_tiles(self) -> usize {
        self.width() * self.width()
    }

    fn layout(self) -> Layout {
        Layout::from_size_align(self.num_tiles(), SCREENBLOCK_SIZE)
            .expect("Failed to create layout, should never happen")
    }

    fn gba_offset(self, pos: Vector2D<i32>) -> usize {
        let x_mod = pos.x & (self.width() as i32 - 1);
        let y_mod = pos.y & (self.width() as i32 - 1);

        let pos = x_mod + (self.width() as i32 * y_mod);

        pos as usize
    }
}

/// Represents a collection of tiles ready to display as an affine background.
///
/// Affine backgrounds work slightly differently to regular backgrounds.
/// You can have at most 2 of them on display at once, and they can only use 256-colour tiles.
/// Also, no per-tile transformations are possible.
/// Finally, only 256 distinct tiles can be used at once across all affine backgrounds.
///
/// Note that while this is in scope, space in
/// the GBA's VRAM will be allocated and unavailable for other backgrounds. You should use the
/// smallest [`AffineBackgroundSize`] you can while still being able to render the scene you want.
///
/// You can show up to 2 affine backgrounds at once (or 1 affine background and 2 [regular backgrounds](super::RegularBackground)).
///
/// to display a given affine background to the screen, you need to call its [show()](AffineBackground::show()) method on
/// a given [`GraphicsFrame`](crate::display::GraphicsFrame).
///
/// ## Example
///
/// ```rust,no_run
/// # #![no_main]
/// # #![no_std]
/// #
/// # use agb::Gba;
/// # fn foo(gba: &mut Gba) {
/// use agb::display::{
///     Priority,
///     tiled::{AffineBackground, AffineBackgroundSize, AffineBackgroundWrapBehaviour, VRAM_MANAGER},
/// };
///
/// let mut gfx = gba.graphics.get();
///
/// let bg = AffineBackground::new(
///     Priority::P0,
///     AffineBackgroundSize::Background16x16,
///     AffineBackgroundWrapBehaviour::NoWrap,
/// );
///
/// // load the background with some tiles
///
/// loop {
///     let mut frame = gfx.frame();
///     bg.show(&mut frame);
///     frame.commit();
/// }
/// # }
/// ```
pub struct AffineBackground {
    priority: Priority,

    tiles: Tiles<u8>,
    screenblock: Rc<AffineBackgroundScreenBlock>,

    scroll: Vector2D<Num<i32, 8>>,

    transform: AffineMatrixBackground,
    wrap_behaviour: AffineBackgroundWrapBehaviour,
}

impl AffineBackground {
    /// Create a new affine background with given `priority`, `size` and `warp_behaviour`.
    ///
    /// This allocates some space in VRAM to store the actual tile data, but doesn't show anything
    /// until you call the [`show()`](AffineBackground::show()) function on a [`GraphicsFrame`](crate::display::GraphicsFrame).
    ///
    /// You can have more `AffineBackground` instances then there are available backgrounds
    /// to show at once, but you can only show 2 at once in a given frame (or 1 and a up to 2
    /// [regular backgrounds](super::RegularBackground)).
    ///
    /// For [`Priority`], a higher priority is rendered first, so is behind lower priorities.
    /// Therefore, `P0` will be rendered at the _front_ and `P3` at the _back_. For equal priorities,
    /// backgrounds are rendered _behind_ objects.
    #[must_use]
    pub fn new(
        priority: Priority,
        size: AffineBackgroundSize,
        wrap_behaviour: AffineBackgroundWrapBehaviour,
    ) -> Self {
        Self {
            priority,

            tiles: Tiles::new(size.num_tiles(), TileFormat::EightBpp),

            scroll: Vector2D::default(),

            screenblock: Rc::new(AffineBackgroundScreenBlock::new(size)),

            transform: AffineMatrixBackground::default(),
            wrap_behaviour,
        }
    }

    /// Set the current scroll position.
    ///
    /// Returns self so you can chain with other `set_` calls.
    pub fn set_scroll_pos(&mut self, scroll: impl Into<Vector2D<Num<i32, 8>>>) -> &mut Self {
        self.scroll = scroll.into();
        self
    }

    /// Get the current scroll position.
    #[must_use]
    pub fn scroll_pos(&self) -> Vector2D<Num<i32, 8>> {
        self.scroll
    }

    /// Set a tile at a given position to the given tile index.
    ///
    /// Because of limitations of the Game Boy Advance, this does not take a [`TileSetting`](super::TileSetting)
    /// but instead just a tile index.
    ///
    /// The `tileset` must also be a 256 colour background imported with the 256 option in
    /// [`include_background_gfx!`](crate::include_background_gfx!).
    ///
    /// This will resulting in copying the tile data to video RAM. However, setting the same tile across multiple locations
    /// in the background will reference that same tile only once to reduce video RAM usage.
    ///
    /// Returns self so you can chain with other `set_` calls.
    pub fn set_tile(
        &mut self,
        pos: impl Into<Vector2D<i32>>,
        tileset: &TileSet<'_>,
        tile_index: u16,
    ) -> &mut Self {
        assert_eq!(
            tileset.format(),
            TileFormat::EightBpp,
            "Can only use 256 colour tiles in an affine background"
        );

        let pos = self.screenblock.size().gba_offset(pos.into());
        self.set_tile_at_pos(pos, tileset, tile_index);

        self
    }

    /// Set the current transformation matrix.
    ///
    /// Returns self so you can chain with other `set_` calls.
    pub fn set_transform(&mut self, transform: impl Into<AffineMatrixBackground>) -> &mut Self {
        self.transform = transform.into();
        self
    }

    /// Get the current transformation matrix.
    #[must_use]
    pub fn transform(&self) -> AffineMatrixBackground {
        self.transform
    }

    /// Set the wrapping behaviour.
    ///
    /// Returns self so you can chain with other `set_` calls.
    pub fn set_wrap_behaviour(
        &mut self,
        wrap_behaviour: AffineBackgroundWrapBehaviour,
    ) -> &mut Self {
        self.wrap_behaviour = wrap_behaviour;
        self
    }

    /// Gets the wrapping behaviour.
    #[must_use]
    pub fn wrap_behaviour(&self) -> AffineBackgroundWrapBehaviour {
        self.wrap_behaviour
    }

    fn set_tile_at_pos(&mut self, pos: usize, tileset: &TileSet<'_>, tile_index: u16) -> &mut Self {
        let old_tile = self.tiles.get(pos);

        let new_tile = if tile_index != TRANSPARENT_TILE_INDEX {
            let new_tile_idx = VRAM_MANAGER.add_tile(tileset, tile_index, true);
            if new_tile_idx.raw_index() > u8::MAX as u16 {
                VRAM_MANAGER.remove_tile(new_tile_idx);
                0
            } else {
                new_tile_idx.raw_index() as u8
            }
        } else {
            0
        };

        if old_tile != 0 {
            VRAM_MANAGER.remove_tile(TileIndex::EightBpp(old_tile as u16));
        }

        if old_tile != new_tile {
            self.tiles.set_tile(pos, new_tile);
        }

        self
    }

    /// Show this background on the given frame.
    ///
    /// The background itself won't be visible until you call [`commit()`](GraphicsFrame::commit()) on the provided [`GraphicsFrame`].
    ///
    /// After this call, you can safely drop the background and the provided [`GraphicsFrame`] will maintain the
    /// references needed until the frame is drawn on screen.
    ///
    /// Note that after this call, any modifications made to the background will _not_ show this frame. Effectively
    /// calling `show()` takes a snapshot of the current state of the background, so you can even modify
    /// the background and `show()` it again and both will show in the frame.
    ///
    /// Returns an [`AffineBackgroundId`] which can be used if you want to apply any additional effects to the background
    /// such as applying [dma effects](crate::dma).
    pub fn show(&self, frame: &mut GraphicsFrame<'_>) -> AffineBackgroundId {
        let commit_data = if self.tiles.is_dirty(self.screenblock.ptr()) {
            Some(AffineBackgroundCommitData {
                tiles: self.tiles.clone(),
                screenblock: Rc::clone(&self.screenblock),
            })
        } else {
            None
        };

        frame.bg_frame.set_next_affine(AffineBackgroundData {
            bg_ctrl: self.bg_ctrl(),
            scroll_offset: self.scroll,
            affine_transform: self.transform,
            commit_data,
        })
    }

    fn bg_ctrl(&self) -> BackgroundControlRegister {
        let mut background_control_register = BackgroundControlRegister::default();

        background_control_register.set_priority(self.priority.into());
        background_control_register
            .set_screen_base_block(u5::new(self.screenblock.screen_base_block() as u8));
        background_control_register.set_overflow_behaviour(self.wrap_behaviour.into());
        background_control_register.set_screen_size(self.screenblock.size().into());

        background_control_register
    }

    /// Returns the size of the affine background.
    #[must_use]
    pub fn size(&self) -> AffineBackgroundSize {
        self.screenblock.size()
    }

    /// Set the current priority for the background.
    ///
    /// This won't take effect until the next time you call [`show()`](AffineBackground::show()).
    /// Returns self so you can chain with other `set_` calls.
    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        self.priority = priority;
        self
    }

    /// Gets the current priority for the background.
    #[must_use]
    pub fn priority(&self) -> Priority {
        self.priority
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C, packed(4))]
#[allow(missing_docs)]
/// An affine matrix that can be used in affine backgrounds
pub struct AffineMatrixBackground {
    pub a: Num<i16, 8>,
    pub b: Num<i16, 8>,
    pub c: Num<i16, 8>,
    pub d: Num<i16, 8>,
    pub x: Num<i32, 8>,
    pub y: Num<i32, 8>,
}

impl Default for AffineMatrixBackground {
    fn default() -> Self {
        Self::from_affine::<i16, 8>(AffineMatrix::identity())
    }
}

impl From<AffineMatrix> for AffineMatrixBackground {
    fn from(value: AffineMatrix) -> Self {
        Self::from_affine(value)
    }
}

impl AffineMatrixBackground {
    /// Converts the matrix to one which can be used in affine backgrounds
    /// wrapping any value which is too large to be represented there.
    #[must_use]
    pub fn from_affine<I, const N: usize>(affine: AffineMatrix<Num<I, N>>) -> Self
    where
        I: FixedWidthSignedInteger,
        i32: From<I>,
    {
        let a: Num<i32, 8> = affine.a.change_base();
        let b: Num<i32, 8> = affine.b.change_base();
        let c: Num<i32, 8> = affine.c.change_base();
        let d: Num<i32, 8> = affine.d.change_base();

        Self {
            a: Num::from_raw(a.to_raw() as i16),
            b: Num::from_raw(b.to_raw() as i16),
            c: Num::from_raw(c.to_raw() as i16),
            d: Num::from_raw(d.to_raw() as i16),
            x: affine.x.change_base(),
            y: affine.y.change_base(),
        }
    }

    #[must_use]
    /// Converts to the affine matrix that is usable in performing efficient
    /// calculations.
    pub fn to_affine_matrix(&self) -> AffineMatrix {
        AffineMatrix {
            a: self.a.change_base(),
            b: self.b.change_base(),
            c: self.c.change_base(),
            d: self.d.change_base(),
            x: self.x,
            y: self.y,
        }
    }

    #[must_use]
    /// Creates a transformation matrix using GBA specific syscalls.
    /// This can be done using the standard transformation matrices like
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// # use agb_fixnum::{Vector2D, Num};
    /// use agb::display::AffineMatrix;
    /// # fn from_scale_rotation_position(
    /// #     transform_origin: Vector2D<Num<i32, 8>>,
    /// #     scale: Vector2D<Num<i32, 8>>,
    /// #     rotation: Num<i32, 8>,
    /// #     position: Vector2D<Num<i32, 8>>,
    /// # ) {
    /// let A = AffineMatrix::from_translation(-transform_origin)
    ///     * AffineMatrix::from_scale(scale)
    ///     * AffineMatrix::from_rotation(rotation)
    ///     * AffineMatrix::from_translation(position);
    /// # }
    /// ```
    pub fn from_scale_rotation_position(
        transform_origin: impl Into<Vector2D<Num<i32, 8>>>,
        scale: impl Into<Vector2D<Num<i32, 8>>>,
        rotation: Num<i32, 16>,
        position: impl Into<Vector2D<i16>>,
    ) -> Self {
        crate::syscall::bg_affine_matrix(
            transform_origin.into(),
            position.into(),
            scale.into().try_change_base().unwrap(),
            rotation.rem_euclid(1.into()).try_change_base().unwrap(),
        )
    }
}

impl From<AffineMatrixBackground> for AffineMatrix {
    fn from(mat: AffineMatrixBackground) -> Self {
        mat.to_affine_matrix()
    }
}
