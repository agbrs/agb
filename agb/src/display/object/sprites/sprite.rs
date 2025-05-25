use core::alloc::Layout;

use crate::display::palette16::Palette16;

use super::{BYTES_PER_TILE_4BPP, BYTES_PER_TILE_8BPP, SpriteVram};

/// Sprite data. Refers to the palette, pixel data, and the size of the sprite.
pub struct Sprite {
    pub(crate) palette: Palette,
    pub(crate) data: &'static [u8],
    pub(crate) size: Size,
}

#[derive(Clone, Copy)]
pub enum Palette {
    Single(&'static Palette16),
    Multi(&'static PaletteMulti),
}

impl Palette {
    pub(crate) fn is_multi(self) -> bool {
        matches!(self, Palette::Multi(_))
    }
}

/// A palette for 256 colour mode.
pub struct PaletteMulti {
    first_index: u32,
    palettes: &'static [Palette16],
}

impl PaletteMulti {
    #[must_use]
    /// Create a new palette. The first index is the index where the palette starts.
    pub const fn new(first_index: u32, palettes: &'static [Palette16]) -> Self {
        assert!(palettes.len() <= 16);
        assert!(!palettes.is_empty());
        assert!(16 - palettes.len() >= first_index as usize);

        Self {
            first_index,
            palettes,
        }
    }
    #[must_use]
    /// Gets the palettes, usually for coping to palette vram.
    pub const fn palettes(&self) -> &'static [Palette16] {
        self.palettes
    }

    #[must_use]
    /// Gets the first index of the palette. When copied to palette vram it is
    /// expected to be copied starting from this index.
    pub const fn first_index(&self) -> u32 {
        self.first_index
    }
}

impl Sprite {
    #[doc(hidden)]
    /// Creates a sprite from it's constituent data, used internally by
    /// [include_aseprite] and should generally not be used outside it.
    ///
    /// # Safety
    /// The data should be aligned to a 2 byte boundary
    #[must_use]
    pub const unsafe fn new(palette: &'static Palette16, data: &'static [u8], size: Size) -> Self {
        Self {
            palette: Palette::Single(palette),
            data,
            size,
        }
    }

    #[doc(hidden)]
    /// Creates a sprite that uses multiple palettes, this will use 256 colour
    /// mode, but can use fewer palettes. The palette location in palette vram
    /// is currently fixed by `first_index`.
    ///
    /// # Safety
    /// The data should be aligned to a 2 byte boundary
    #[must_use]
    pub const unsafe fn new_multi(
        palettes: &'static PaletteMulti,
        data: &'static [u8],
        size: Size,
    ) -> Self {
        Self {
            palette: Palette::Multi(palettes),
            data,
            size,
        }
    }

    #[must_use]
    /// Gives the size of the sprite
    pub fn size(&self) -> Size {
        self.size
    }
}

/// The sizes of sprite supported by the GBA.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[allow(missing_docs)]
pub enum Size {
    // stored as attr0 attr1
    S8x8 = 0b00_00,
    S16x16 = 0b00_01,
    S32x32 = 0b00_10,
    S64x64 = 0b00_11,

    S16x8 = 0b01_00,
    S32x8 = 0b01_01,
    S32x16 = 0b01_10,
    S64x32 = 0b01_11,

    S8x16 = 0b10_00,
    S8x32 = 0b10_01,
    S16x32 = 0b10_10,
    S32x64 = 0b10_11,
}

#[doc(hidden)]
#[macro_export]
macro_rules! align_bytes {
    ($align_ty:ty, $data:literal) => {{
        #[repr(C)] // guarantee 'bytes' comes after '_align'
        struct AlignedAs<Align, Bytes: ?Sized> {
            pub _align: [Align; 0],
            pub bytes: Bytes,
        }

        const ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
            _align: [],
            bytes: *$data,
        };

        &ALIGNED.bytes
    }};
}

/// Includes sprites found in the referenced aseprite files.
///
/// Can include multiple at once and optimises palettes of all included in the
/// single call together. See [Size] for supported sizes.
///
/// This generates a module given by the first argument, you can control the
/// visibility of the module using the normal means. The generated module
/// exports each Tag in the aseprite file as a static, the static will be all
/// caps and have spaces and dashes converted to underscores.
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// use agb::include_aseprite;
/// include_aseprite!(
///     mod sprites,
///     "examples/gfx/chicken.aseprite"
/// );
///
/// use sprites::{JUMP, WALK, IDLE};
/// ```
/// The tags from the aseprite file are included so you can refer to sprites by
/// name in code. You should ensure tags are unique as this is not enforced by
/// aseprite.
///
/// Including from the out directory is supported through the `$OUT_DIR` token.
///
/// ```rust,ignore
/// # #![no_std]
/// # #![no_main]
/// use agb::include_aseprite;
/// include_aseprite!(
///     mod sprites,
///     "$OUT_DIR/generated_sprite.aseprite"
/// );
/// ```
///
/// You may pass multiple aseprite files in. This is particularly useful if you
/// have multiple sprites with different sizes since aseprite files require
/// that every frame has the same size.
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// use agb::include_aseprite;
/// include_aseprite!(
///     mod sprites,
///     "examples/gfx/crab.aseprite",
///     "examples/gfx/crab-small.aseprite"
/// );
/// ```
#[macro_export]
macro_rules! include_aseprite {
    ($v: vis mod $module: ident, $($aseprite_path: expr),*$(,)?) => {
        $v mod $module {
            #[allow(unused_imports)]
            use $crate::display::object::{Size, Sprite, Tag};
            use $crate::display::{Palette16, Rgb15};
            use $crate::align_bytes;

            $crate::include_aseprite_inner!($($aseprite_path),*);
        }
    };
}

/// Includes sprites found in the referenced aseprite files.
///
/// This will optimise to a single multi palette, 256 colour sprites.
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// use agb::include_aseprite_256;
/// include_aseprite_256!(
///     mod sprites,
///     "examples/gfx/chicken.aseprite"
/// );
///
/// use sprites::{JUMP, WALK, IDLE};
/// ```
#[macro_export]
macro_rules! include_aseprite_256 {
    ($v: vis mod $module: ident, $($aseprite_path: expr),*$(,)?) => {
        $v mod $module {
            #[allow(unused_imports)]
            use $crate::display::object::{Size, Sprite, Tag, PaletteMulti};
            use $crate::display::{Palette16, Rgb15};
            use $crate::align_bytes;

            $crate::include_aseprite_256_inner!($($aseprite_path),*);
        }
    }
}

pub use include_aseprite;

#[derive(Clone, Copy)]
enum Direction {
    Forward,
    Backward,
    PingPong,
}

impl Direction {
    const fn from_usize(a: usize) -> Self {
        match a {
            0 => Direction::Forward,
            1 => Direction::Backward,
            2 => Direction::PingPong,
            _ => panic!("Invalid direction, this is a bug in image converter or agb"),
        }
    }
}

/// A sequence of sprites from aseprite.
pub struct Tag {
    sprites: &'static [Sprite],
    direction: Direction,
}

/// An [`AnimationIterator`] that repeats each frame of the animation a certain
/// number of times. It is created by the [`AnimationIterator::repeat`] method.
pub struct RepeatingAnimationIterator {
    remaining: u16,
    repetitions: u16,
    inner: AnimationIterator,
}

impl Iterator for RepeatingAnimationIterator {
    type Item = SpriteVram;

    fn next(&mut self) -> Option<Self::Item> {
        self.remaining -= 1;
        if self.remaining == 0 {
            self.remaining = self.repetitions;
            self.inner.next()
        } else {
            Some(self.inner.peek())
        }
    }
}

/// An infinite iterator over the frames of the animation
#[derive(Clone)]
pub struct AnimationIterator {
    cached_image: SpriteVram,
    frame: i32,
    tag: &'static Tag,
}

impl AnimationIterator {
    #[must_use]
    /// Gives the current sprite from the animation iterator
    pub fn peek(&self) -> SpriteVram {
        self.cached_image.clone()
    }

    #[must_use]
    /// Repeats each frame of the animation times times
    ///
    /// # Panics
    /// Panics if the number of times to repeat is zero
    pub fn repeat(self, times: u16) -> RepeatingAnimationIterator {
        assert!(times > 0);
        RepeatingAnimationIterator {
            remaining: times,
            repetitions: times,
            inner: self,
        }
    }
}

impl Iterator for AnimationIterator {
    type Item = SpriteVram;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.frame;

        let sprite = match self.tag.direction {
            Direction::Forward => {
                self.frame += 1;
                if self.frame >= self.tag.sprites.len() as i32 {
                    self.frame = 0;
                }
                self.tag.sprite(current as usize)
            }
            Direction::Backward => {
                self.frame -= 1;
                if self.frame < 0 {
                    self.frame = self.tag.sprites.len() as i32 - 1;
                }
                self.tag.sprite(current as usize)
            }
            Direction::PingPong => {
                self.frame += 1;
                if self.frame >= self.tag.sprites.len() as i32 * 2 - 2 {
                    self.frame = 0;
                }

                if current >= self.tag.sprites.len() as i32 {
                    let idx = self.tag.sprites.len() * 2 - current as usize - 2;
                    self.tag.sprite(idx)
                } else {
                    self.tag.sprite(current as usize)
                }
            }
        };

        self.cached_image = sprite.into();
        Some(self.cached_image.clone())
    }
}

unsafe impl Sync for Tag {}

impl Tag {
    /// The individual sprites that make up the animation themselves.
    #[must_use]
    pub fn sprites(&self) -> &'static [Sprite] {
        self.sprites
    }

    /// A single sprite referred to by index in the animation sequence.
    #[must_use]
    pub const fn sprite(&self, idx: usize) -> &'static Sprite {
        &self.sprites[idx]
    }

    /// A sprite that follows the animation sequence. For instance, in aseprite
    /// tags can be specified to animate:
    /// * Forward
    /// * Backward
    /// * Ping pong
    ///
    /// This takes the animation type in account and returns the correct sprite
    /// following these requirements.
    #[inline]
    #[must_use]
    pub fn animation_sprite(&self, idx: usize) -> &'static Sprite {
        let len_sub_1 = self.sprites.len() - 1;
        match self.direction {
            Direction::Forward => self.sprite(idx % self.sprites.len()),
            Direction::Backward => self.sprite(len_sub_1 - (idx % self.sprites.len())),
            Direction::PingPong => self.sprite(
                (((idx + len_sub_1) % (len_sub_1 * 2)) as isize - len_sub_1 as isize)
                    .unsigned_abs(),
            ),
        }
    }

    #[must_use]
    /// An iterator over the frames of the animation iterator. This is more
    /// efficient than calling [`Self::animation_sprite`] due to not using a
    /// divide operation but does mean you may need to keep track of more
    /// information
    pub fn iter(&'static self) -> AnimationIterator {
        AnimationIterator {
            cached_image: self.sprite(0).into(),
            frame: match self.direction {
                Direction::Forward | Direction::PingPong => 0,
                Direction::Backward => self.sprites.len() as i32 - 1,
            },
            tag: self,
        }
    }

    #[doc(hidden)]
    /// Creates a new sprite from it's constituent parts. Used internally by
    /// [include_aseprite] and should generally not be used elsewhere.
    #[must_use]
    pub const fn new(sprites: &'static [Sprite], direction: usize) -> Self {
        Self {
            sprites,
            direction: Direction::from_usize(direction),
        }
    }
}

impl Size {
    pub(crate) const fn number_of_tiles(self) -> usize {
        match self {
            Size::S8x8 => 1,
            Size::S16x16 => 4,
            Size::S32x32 => 16,
            Size::S64x64 => 64,
            Size::S16x8 => 2,
            Size::S32x8 => 4,
            Size::S32x16 => 8,
            Size::S64x32 => 32,
            Size::S8x16 => 2,
            Size::S8x32 => 4,
            Size::S16x32 => 8,
            Size::S32x64 => 32,
        }
    }
    pub(crate) const fn shape_size(self) -> (u16, u16) {
        (self as u16 >> 2, self as u16 & 0b11)
    }

    pub(crate) fn layout(self, multi_palette: bool) -> Layout {
        Layout::from_size_align(
            self.number_of_tiles() * BYTES_PER_TILE_4BPP * (multi_palette as usize + 1),
            8,
        )
        .unwrap()
    }

    /// The size in bytes for a sprite of this size in 16 colour mode
    #[must_use]
    pub fn size_bytes_16(self) -> usize {
        self.number_of_tiles() * BYTES_PER_TILE_4BPP
    }

    /// The size in bytes for a sprite of this size in 256 colour mode
    #[must_use]
    pub fn size_bytes_256(self) -> usize {
        self.number_of_tiles() * BYTES_PER_TILE_8BPP
    }

    #[must_use]
    /// Creates a size from width and height in pixels, panics if the width and
    /// height is not representable by GBA sprites.
    ///
    /// # Panics
    /// Panics if the given size is not representable by the GBA
    pub const fn from_width_height(width: usize, height: usize) -> Self {
        match (width, height) {
            (8, 8) => Size::S8x8,
            (16, 16) => Size::S16x16,
            (32, 32) => Size::S32x32,
            (64, 64) => Size::S64x64,
            (16, 8) => Size::S16x8,
            (32, 8) => Size::S32x8,
            (32, 16) => Size::S32x16,
            (64, 32) => Size::S64x32,
            (8, 16) => Size::S8x16,
            (8, 32) => Size::S8x32,
            (16, 32) => Size::S16x32,
            (32, 64) => Size::S32x64,
            (_, _) => panic!("Bad width and height!"),
        }
    }

    #[must_use]
    /// Returns the width and height of the size in pixels.
    pub const fn to_width_height(self) -> (usize, usize) {
        match self {
            Size::S8x8 => (8, 8),
            Size::S16x16 => (16, 16),
            Size::S32x32 => (32, 32),
            Size::S64x64 => (64, 64),
            Size::S16x8 => (16, 8),
            Size::S32x8 => (32, 8),
            Size::S32x16 => (32, 16),
            Size::S64x32 => (64, 32),
            Size::S8x16 => (8, 16),
            Size::S8x32 => (8, 32),
            Size::S16x32 => (16, 32),
            Size::S32x64 => (32, 64),
        }
    }

    #[must_use]
    /// Returns the width and height of the size in pixels.
    pub const fn to_tiles_width_height(self) -> (usize, usize) {
        let wh = self.to_width_height();
        (wh.0 / 8, wh.1 / 8)
    }
}

#[cfg(test)]
mod tests {

    use crate::display::Rgb15;

    use super::*;

    const DUMMY_SPRITE: Sprite = Sprite {
        palette: Palette::Single(&Palette16 {
            colours: [Rgb15(0); 16],
        }),
        data: &[],
        size: Size::S16x16,
    };

    macro_rules! create_iterator {
        ($length: literal, $direction: expr ) => {{
            const SPRITES: &[Sprite] = &[DUMMY_SPRITE; $length];
            const TAG: &Tag = &Tag {
                sprites: SPRITES,
                direction: $direction,
            };

            AnimationIterator {
                cached_image: TAG.sprite(0).into(),
                frame: 0,
                tag: &TAG,
            }
        }};
    }

    #[test_case]
    fn check_forward(_: &mut crate::Gba) {
        let mut iterator = create_iterator!(3, Direction::Forward);

        let mut assert_is_at_idx = |idx: usize| {
            iterator.next();
            assert_eq!(iterator.frame, idx as i32);
        };

        assert_is_at_idx(1);
        assert_is_at_idx(2);
        assert_is_at_idx(0);
        assert_is_at_idx(1);
        assert_is_at_idx(2);
        assert_is_at_idx(0);
    }

    #[test_case]
    fn check_backward(_: &mut crate::Gba) {
        let mut iterator = create_iterator!(3, Direction::Backward);

        let mut assert_is_at_idx = |idx: usize| {
            iterator.next();
            assert_eq!(iterator.frame, idx as i32);
        };

        assert_is_at_idx(2);
        assert_is_at_idx(1);
        assert_is_at_idx(0);
        assert_is_at_idx(2);
        assert_is_at_idx(1);
        assert_is_at_idx(0);
    }

    #[test_case]
    fn check_pingpong(_: &mut crate::Gba) {
        let mut iterator = create_iterator!(3, Direction::PingPong);

        let mut assert_is_at_idx = |idx: usize| {
            iterator.next();
            assert_eq!(iterator.frame, idx as i32);
        };

        assert_is_at_idx(1);
        assert_is_at_idx(2);
        assert_is_at_idx(3);
        assert_is_at_idx(0);
        assert_is_at_idx(1);
        assert_is_at_idx(2);
        assert_is_at_idx(3);
        assert_is_at_idx(0);
    }

    #[test_case]
    fn check_repeating(_: &mut crate::Gba) {
        let iterator = create_iterator!(3, Direction::Forward);

        let mut iterator = iterator.repeat(3);

        let mut assert_is_at_idx = |idx: usize| {
            iterator.next();
            assert_eq!(iterator.inner.frame, idx as i32);
        };

        assert_is_at_idx(0);
        assert_is_at_idx(0);

        assert_is_at_idx(1);
        assert_is_at_idx(1);
        assert_is_at_idx(1);

        assert_is_at_idx(2);
        assert_is_at_idx(2);
        assert_is_at_idx(2);

        assert_is_at_idx(0);
        assert_is_at_idx(0);
        assert_is_at_idx(0);

        assert_is_at_idx(1);
        assert_is_at_idx(1);
        assert_is_at_idx(1);

        assert_is_at_idx(2);
        assert_is_at_idx(2);
        assert_is_at_idx(2);
    }
}
