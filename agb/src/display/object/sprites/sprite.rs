use core::{alloc::Layout, slice};

use crate::display::palette16::Palette16;

use super::BYTES_PER_TILE_4BPP;

/// Sprite data. Refers to the palette, pixel data, and the size of the sprite.
pub struct Sprite {
    pub(crate) palette: &'static Palette16,
    pub(crate) data: &'static [u8],
    pub(crate) size: Size,
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
            palette,
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

/// Includes sprites found in the referenced aseprite files. Can include
/// multiple at once and optimises palettes of all included in the single call
/// together. See [Size] for supported sizes. Returns a reference to [Graphics].
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// # use agb::{display::object::Graphics, include_aseprite};
/// const GRAPHICS: &Graphics = include_aseprite!(
///     "examples/gfx/boss.aseprite",
///     "examples/gfx/objects.aseprite"
/// );
/// ```
/// The tags from the aseprite file are included so you can refer to sprites by
/// name in code. You should ensure tags are unique as this is not enforced by
/// aseprite.
///
#[macro_export]
macro_rules! include_aseprite {
    ($($aseprite_path: expr),*) => {{
        use $crate::display::object::{Size, Sprite, Tag, TagMap, Graphics};
        use $crate::display::palette16::Palette16;
        use $crate::align_bytes;

        $crate::include_aseprite_inner!($($aseprite_path),*);

        &Graphics::new(SPRITES, TAGS)
    }};
}

pub use include_aseprite;

/// Stores sprite and tag data returned by [include_aseprite].
pub struct Graphics {
    sprites: &'static [Sprite],
    tag_map: &'static TagMap,
}

impl Graphics {
    #[doc(hidden)]
    /// Creates graphics data from sprite data and a tag_map. This is used
    /// internally by [include_aseprite] and would be otherwise difficult to
    /// use.
    #[must_use]
    pub const fn new(sprites: &'static [Sprite], tag_map: &'static TagMap) -> Self {
        Self { sprites, tag_map }
    }
    #[must_use]
    /// Gets the tag map from the aseprite files. This allows reference to
    /// sprite sequences by name.
    pub const fn tags(&self) -> &TagMap {
        self.tag_map
    }
    /// Gets a big list of the sprites themselves. Using tags is often easier.
    #[must_use]
    pub const fn sprites(&self) -> &[Sprite] {
        self.sprites
    }
}

/// Stores aseprite tags. Can be used to refer to animation sequences by name.
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// # use agb::{display::object::{Graphics, Tag}, include_aseprite};
/// const GRAPHICS: &Graphics = include_aseprite!(
///     "examples/gfx/boss.aseprite",
///     "examples/gfx/objects.aseprite"
/// );
///
/// const EMU_WALK: &Tag = GRAPHICS.tags().get("emu-walk");
/// ```
/// This being the whole animation associated with the walk sequence of the emu.
/// See [Tag] for details on how to use this.
pub struct TagMap {
    tags: &'static [(&'static str, Tag)],
}

const fn const_byte_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

impl TagMap {
    #[doc(hidden)]
    /// Creates a new tag map from (name, Tag) pairs. Used internally by
    /// [include_aseprite] and should not really be used outside of it.
    #[must_use]
    pub const fn new(tags: &'static [(&'static str, Tag)]) -> TagMap {
        Self { tags }
    }

    #[doc(hidden)]
    /// Attempts to get a tag. Generally should not be used.
    #[must_use]
    pub const fn try_get(&'static self, tag: &str) -> Option<&'static Tag> {
        let mut i = 0;
        while i < self.tags.len() {
            let s = self.tags[i].0;
            if const_byte_compare(s.as_bytes(), tag.as_bytes()) {
                return Some(&self.tags[i].1);
            }

            i += 1;
        }

        None
    }

    /// Gets a tag associated with the name. A tag in aseprite refers to a
    /// sequence of sprites with some metadata for how to animate it. You should
    /// call this in a constant context so it is evaluated at compile time. It
    /// is inefficient to call this elsewhere.
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// # use agb::{display::object::{Graphics, Tag}, include_aseprite};
    /// const GRAPHICS: &Graphics = include_aseprite!(
    ///     "examples/gfx/boss.aseprite",
    ///     "examples/gfx/objects.aseprite"
    /// );
    ///
    /// const EMU_WALK: &Tag = GRAPHICS.tags().get("emu-walk");
    /// ```
    ///
    /// See [Tag] for more details.
    #[must_use]
    pub const fn get(&'static self, tag: &str) -> &'static Tag {
        let t = self.try_get(tag);
        match t {
            Some(t) => t,
            None => panic!("The requested tag does not exist"),
        }
    }

    /// Takes an iterator over all the tags in the map. Not generally useful.
    pub fn values(&self) -> impl Iterator<Item = &'static Tag> {
        self.tags.iter().map(|x| &x.1)
    }
}

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
    sprites: *const Sprite,
    len: usize,
    direction: Direction,
}

impl Tag {
    /// The individual sprites that make up the animation themselves.
    #[must_use]
    pub fn sprites(&self) -> &'static [Sprite] {
        unsafe { slice::from_raw_parts(self.sprites, self.len) }
    }

    /// A single sprite referred to by index in the animation sequence.
    #[must_use]
    pub const fn sprite(&self, idx: usize) -> &'static Sprite {
        if idx >= self.len {
            panic!("out of bounds access to sprite");
        }
        unsafe { &*self.sprites.add(idx) }
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
        let len_sub_1 = self.len - 1;
        match self.direction {
            Direction::Forward => self.sprite(idx % self.len),
            Direction::Backward => self.sprite(len_sub_1 - (idx % self.len)),
            Direction::PingPong => self.sprite(
                (((idx + len_sub_1) % (len_sub_1 * 2)) as isize - len_sub_1 as isize)
                    .unsigned_abs(),
            ),
        }
    }

    #[doc(hidden)]
    /// Creates a new sprite from it's constituent parts. Used internally by
    /// [include_aseprite] and should generally not be used elsewhere.
    #[must_use]
    pub const fn new(sprites: &'static [Sprite], from: usize, to: usize, direction: usize) -> Self {
        assert!(from <= to);
        assert!(to < sprites.len());
        Self {
            sprites: &sprites[from] as *const Sprite,
            len: to - from + 1,
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

    pub(crate) fn layout(self) -> Layout {
        Layout::from_size_align(self.number_of_tiles() * BYTES_PER_TILE_4BPP, 8).unwrap()
    }

    #[must_use]
    /// Creates a size from width and height in pixels, panics if the width and
    /// height is not representable by GBA sprites.
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
}
