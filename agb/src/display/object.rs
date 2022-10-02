#![deny(missing_docs)]
use alloc::rc::{Rc, Weak};
use alloc::vec::Vec;
use core::alloc::Layout;

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::DerefMut;
use core::ptr::NonNull;
use core::slice;
use modular_bitfield::prelude::{B10, B2, B3, B4, B5, B8, B9};
use modular_bitfield::{bitfield, BitfieldSpecifier};

const BYTES_PER_TILE_4BPP: usize = 32;

use super::palette16::Palette16;
use super::{Priority, DISPLAY_CONTROL};
use crate::agb_alloc::block_allocator::BlockAllocator;
use crate::agb_alloc::bump_allocator::StartEnd;
use crate::dma;
use crate::fixnum::Vector2D;
use crate::hash_map::HashMap;

use attributes::*;

/// Include this type if you call `get_object_controller` in impl block. This
/// helps you use the right lifetimes and doesn't impl Sync (using from two
/// "threads" without syncronisation is not safe), but sending to another
/// "thread" is safe.
#[derive(Clone, Copy)]
struct ObjectControllerReference<'a> {
    #[cfg(debug_assertions)]
    reference: &'a core::cell::RefCell<ObjectControllerStatic>,

    _ref: PhantomData<&'a UnsafeCell<()>>,
}

#[cfg(debug_assertions)]
static mut OBJECT_CONTROLLER: MaybeUninit<core::cell::RefCell<ObjectControllerStatic>> =
    MaybeUninit::uninit();
#[cfg(not(debug_assertions))]
static mut OBJECT_CONTROLLER: MaybeUninit<ObjectControllerStatic> = MaybeUninit::uninit();

impl<'a> ObjectControllerReference<'a> {
    unsafe fn init() -> Self {
        #[cfg(debug_assertions)]
        OBJECT_CONTROLLER.write(core::cell::RefCell::new(ObjectControllerStatic::new()));
        #[cfg(not(debug_assertions))]
        OBJECT_CONTROLLER.write(ObjectControllerStatic::new());
        Self {
            #[cfg(debug_assertions)]
            reference: unsafe { OBJECT_CONTROLLER.assume_init_ref() },
            _ref: PhantomData,
        }
    }

    unsafe fn uninit() {
        OBJECT_CONTROLLER.assume_init_drop();
    }

    #[track_caller]
    #[cfg(debug_assertions)]
    fn borrow_cell_ref(self) -> core::cell::RefMut<'a, ObjectControllerStatic> {
        self.reference.borrow_mut()
    }
    #[track_caller]
    #[cfg(not(debug_assertions))]
    unsafe fn borrow_direct(self) -> &'a mut ObjectControllerStatic {
        unsafe { OBJECT_CONTROLLER.assume_init_mut() }
    }

    #[track_caller]
    unsafe fn borrow_mut(self) -> impl DerefMut<Target = ObjectControllerStatic> + 'a {
        #[cfg(debug_assertions)]
        {
            self.reference.borrow_mut()
        }
        #[cfg(not(debug_assertions))]
        unsafe {
            OBJECT_CONTROLLER.assume_init_mut()
        }
    }
}

static SPRITE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || TILE_SPRITE,
        end: || TILE_SPRITE + 1024 * 8 * 4,
    })
};

static PALETTE_ALLOCATOR: BlockAllocator = unsafe {
    BlockAllocator::new(StartEnd {
        start: || PALETTE_SPRITE,
        end: || PALETTE_SPRITE + 0x200,
    })
};

const PALETTE_SPRITE: usize = 0x0500_0200;
const TILE_SPRITE: usize = 0x06010000;
const OBJECT_ATTRIBUTE_MEMORY: usize = 0x0700_0000;

/// Sprite data. Refers to the palette, pixel data, and the size of the sprite.
pub struct Sprite {
    palette: &'static Palette16,
    data: &'static [u8],
    size: Size,
}

/// Sprite data that can be used to create sprites in vram.
pub struct DynamicSprite<'a> {
    data: &'a [u8],
    size: Size,
}

impl DynamicSprite<'_> {
    #[must_use]
    /// Creates a new dynamic sprite from underlying bytes. Note that despite
    /// being an array of u8, this must be aligned to at least a 2 byte
    /// boundary.
    pub fn new(data: &[u8], size: Size) -> DynamicSprite {
        let ptr = &data[0] as *const _ as usize;
        if ptr % 2 != 0 {
            panic!("data is not aligned to a 2 byte boundary");
        }
        if data.len() != size.number_of_tiles() * BYTES_PER_TILE_4BPP {
            panic!(
                "data is not of expected length, got {} expected {}",
                data.len(),
                size.number_of_tiles() * BYTES_PER_TILE_4BPP
            );
        }
        DynamicSprite { data, size }
    }

    #[must_use]
    /// Tries to copy the sprite to vram to be used to set object sprites.
    /// Returns None if there is no room in sprite vram.
    pub fn try_vram(&self, palette: PaletteVram) -> Option<SpriteBorrow> {
        Some(SpriteBorrow {
            sprite: unsafe { SpriteVram::new(self.data, self.size, palette)? },
        })
    }

    #[must_use]
    /// Tries to copy the sprite to vram to be used to set object sprites.
    /// Panics if there is no room in sprite vram.
    pub fn to_vram(&self, palette: PaletteVram) -> SpriteBorrow {
        self.try_vram(palette)
            .expect("No slot for sprite available")
    }
}

/// The sizes of sprite supported by the GBA.
#[derive(Clone, Copy, PartialEq, Eq)]
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
#[repr(C)] // guarantee 'bytes' comes after '_align'
pub struct AlignedAs<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
}

#[doc(hidden)]
#[macro_export]
macro_rules! align_bytes {
    ($align_ty:ty, $data:literal) => {{
        use $crate::display::object::AlignedAs;

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
    /// call this in a constant context so it is evalulated at compile time. It
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
    Pingpong,
}

impl Direction {
    const fn from_usize(a: usize) -> Self {
        match a {
            0 => Direction::Forward,
            1 => Direction::Backward,
            2 => Direction::Pingpong,
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

    /// A single sprite refered to by index in the animation sequence.
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
            Direction::Pingpong => self.sprite(
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
    const fn number_of_tiles(self) -> usize {
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
    const fn shape_size(self) -> (u8, u8) {
        (self as u8 >> 2, self as u8 & 0b11)
    }

    fn layout(self) -> Layout {
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

/// A reference to a sprite that is currently copied in vram. This is reference
/// counted and can be cloned to keep it in vram or otherwise manipulated. If
/// objects no longer refer to this sprite, then it's vram slot is freed for the
/// next sprite. This is obtained from the [ObjectController].
#[derive(Clone)]
pub struct SpriteBorrow {
    sprite: SpriteVram,
}

#[derive(PartialEq, Eq)]
struct Attributes {
    a0: ObjectAttribute0,
    a1s: ObjectAttribute1Standard,
    a1a: ObjectAttribute1Affine,
    a2: ObjectAttribute2,
}

impl Attributes {
    fn new() -> Self {
        Self {
            a0: ObjectAttribute0::new(),
            a1s: ObjectAttribute1Standard::new(),
            a1a: ObjectAttribute1Affine::new(),
            a2: ObjectAttribute2::new(),
        }
    }

    fn commit(&self, location: usize) {
        let mode = self.a0.object_mode();
        let attrs: [[u8; 2]; 3] = match mode {
            ObjectMode::Normal => [
                self.a0.into_bytes(),
                self.a1s.into_bytes(),
                self.a2.into_bytes(),
            ],
            _ => [
                self.a0.into_bytes(),
                self.a1a.into_bytes(),
                self.a2.into_bytes(),
            ],
        };

        unsafe {
            let attrs: [u16; 3] = core::mem::transmute(attrs);
            let ptr = (OBJECT_ATTRIBUTE_MEMORY as *mut u16).add(location * 4);

            ptr.add(0).write_volatile(attrs[0]);
            ptr.add(1).write_volatile(attrs[1]);
            ptr.add(2).write_volatile(attrs[2]);
        };
    }
}

/// An object that may be displayed on screen. The object can be modified using
/// the available methods. This is obtained from the [ObjectController].
pub struct Object<'a> {
    loan: Loan<'a>,
}

#[derive(Clone, Copy)]
struct Location(usize);

impl Location {
    fn from_sprite_ptr(d: NonNull<u8>) -> Self {
        Self(((d.as_ptr() as usize) - TILE_SPRITE) / BYTES_PER_TILE_4BPP)
    }
    fn from_palette_ptr(d: NonNull<u8>) -> Self {
        Self((d.as_ptr() as usize - PALETTE_SPRITE) / Palette16::layout().size())
    }
    fn as_palette_ptr(self) -> *mut u8 {
        (self.0 as usize * Palette16::layout().size() + PALETTE_SPRITE) as *mut u8
    }
    fn as_sprite_ptr(self) -> *mut u8 {
        (self.0 as usize * BYTES_PER_TILE_4BPP + TILE_SPRITE) as *mut u8
    }
}

#[derive(Clone)]
/// The palette data in Vram, this is reference counted and the palette data is
/// removed and can be reused from vram when no strong references remain.
pub struct PaletteVram(Rc<PaletteData>);

#[derive(Clone)]
struct SpriteVram(Rc<SpriteArena>);

struct PaletteData {
    location: Location,
}

impl PaletteVram {
    /// Creates a palette in vram from the given palette. Can be used to create
    /// sprites in vram in the [DynamicSprite] functions.
    #[must_use]
    pub fn new(palette: &Palette16) -> Option<Self> {
        let dest = unsafe { PALETTE_ALLOCATOR.alloc(Palette16::layout())? };

        unsafe {
            dma::dma_copy16(
                palette.colours.as_ptr().cast(),
                dest.as_ptr().cast(),
                palette.colours.len(),
            );
        }

        Some(PaletteVram(Rc::new(PaletteData {
            location: Location::from_palette_ptr(dest),
        })))
    }
}

impl SpriteVram {
    /// # Safety
    /// data should be aligned to a 2 byte boundary
    unsafe fn new(data: &[u8], size: Size, palette: PaletteVram) -> Option<Self> {
        let dest = unsafe { SPRITE_ALLOCATOR.alloc(size.layout())? };

        unsafe {
            dma::dma_copy16(data.as_ptr().cast(), dest.as_ptr().cast(), data.len() / 2);
        }

        Some(SpriteVram(Rc::new(SpriteArena {
            location: Location::from_sprite_ptr(dest),
            size,
            palette,
        })))
    }
}

impl Drop for PaletteData {
    fn drop(&mut self) {
        unsafe { PALETTE_ALLOCATOR.dealloc(self.location.as_palette_ptr(), Palette16::layout()) };
    }
}

struct SpriteArena {
    location: Location,
    size: Size,
    palette: PaletteVram,
}

impl Drop for SpriteArena {
    fn drop(&mut self) {
        unsafe { SPRITE_ALLOCATOR.dealloc(self.location.as_sprite_ptr(), self.size.layout()) }
    }
}

#[derive(Default)]
struct SpriteControllerInner {
    static_palette_map: HashMap<PaletteId, Weak<PaletteData>>,
    static_sprite_map: HashMap<SpriteId, Weak<SpriteArena>>,
}

struct Loan<'a> {
    index: u8,
    controller: ObjectControllerReference<'a>,
}

impl Drop for Loan<'_> {
    fn drop(&mut self) {
        let mut s = unsafe { self.controller.borrow_mut() };

        unsafe {
            s.shadow_oam[self.index as usize]
                .as_mut()
                .unwrap_unchecked()
                .destroy = true;
        };
    }
}

struct ObjectInner {
    attrs: Attributes,
    sprite: SpriteBorrow,
    previous_sprite: SpriteBorrow,
    destroy: bool,
    z: i32,
}

struct ObjectControllerStatic {
    _free_affine_matricies: Vec<u8>,
    free_object: Vec<u8>,
    shadow_oam: Vec<Option<ObjectInner>>,
    z_order: Vec<u8>,
    sprite_controller: SpriteControllerInner,
}

impl ObjectControllerStatic {
    unsafe fn new() -> Self {
        Self {
            shadow_oam: (0..128).map(|_| None).collect(),
            z_order: (0..128).collect(),
            free_object: (0..128).collect(),
            _free_affine_matricies: (0..32).collect(),
            sprite_controller: SpriteControllerInner::new(),
        }
    }

    fn update_z_ordering(&mut self) {
        let shadow_oam = &self.shadow_oam;
        self.z_order
            .sort_by_key(|&a| shadow_oam[a as usize].as_ref().map_or(i32::MAX, |s| s.z));
    }
}

/// A controller that distributes objects and sprites. This controls sprites and
/// objects being copied to vram when it needs to be.
pub struct ObjectController {
    inner: ObjectControllerReference<'static>,
}

impl Drop for ObjectController {
    fn drop(&mut self) {
        unsafe {
            ObjectControllerReference::uninit();
        }
    }
}

const HIDDEN_VALUE: u16 = 0b10 << 8;

impl ObjectController {
    /// Commits the objects to vram and delete sprites where possible. This
    /// should be called shortly after having waited for the next vblank to
    /// ensure what is displayed on screen doesn't change part way through.
    pub fn commit(&self) {
        let mut s = unsafe { self.inner.borrow_mut() };

        let s = &mut *s;

        for (i, &z) in s.z_order.iter().enumerate() {
            if let Some(o) = &mut s.shadow_oam[z as usize] {
                if o.destroy {
                    s.free_object.push(z);

                    unsafe {
                        (OBJECT_ATTRIBUTE_MEMORY as *mut u16)
                            .add((i as usize) * 4)
                            .write_volatile(HIDDEN_VALUE);
                    }

                    let _ = unsafe { s.shadow_oam[z as usize].take().unwrap_unchecked() };
                } else {
                    o.attrs.commit(i);
                    o.previous_sprite = o.sprite.clone();
                }
            } else {
                unsafe {
                    (OBJECT_ATTRIBUTE_MEMORY as *mut u16)
                        .add(i * 4)
                        .write_volatile(HIDDEN_VALUE);
                }
            }
        }

        s.sprite_controller.gc();
    }

    pub(crate) fn new() -> Self {
        DISPLAY_CONTROL.set_bits(1, 1, 0x6);
        DISPLAY_CONTROL.set_bits(1, 1, 0xC);
        DISPLAY_CONTROL.set_bits(0, 1, 0x7);

        for i in 0..128 {
            unsafe {
                (OBJECT_ATTRIBUTE_MEMORY as *mut u16)
                    .add(i * 4)
                    .write_volatile(HIDDEN_VALUE);
            }
        }

        Self {
            inner: unsafe { ObjectControllerReference::init() },
        }
    }

    #[must_use]
    /// Creates an object with it's initial sprite being the sprite reference.
    /// Panics if there is no space for the sprite or if there are no free
    /// objects. This will reuse an existing copy of the sprite in vram if
    /// possible.
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
    ///
    /// # fn foo(gba: &mut agb::Gba) {
    /// # let object_controller = gba.display.object.get();
    /// let emu = object_controller.object_sprite(EMU_WALK.animation_sprite(0));
    /// # }
    /// ```
    pub fn object_sprite<'a>(&'a self, sprite: &'static Sprite) -> Object<'a> {
        let sprite = self.sprite(sprite);
        self.object(sprite)
    }

    #[must_use]
    /// Creates an object with it's initial sprite being the sprite reference.
    /// Returns [None] if the sprite or object could not be allocated. This will
    /// reuse an existing copy of the sprite in vram if possible.
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
    ///
    /// # fn foo(gba: &mut agb::Gba) {
    /// # let object_controller = gba.display.object.get();
    /// let emu = object_controller.try_get_object_sprite(
    ///     EMU_WALK.animation_sprite(0)
    /// ).expect("the sprite or object could be allocated");
    /// # }
    /// ```
    pub fn try_get_object_sprite<'a>(&'a self, sprite: &'static Sprite) -> Option<Object<'a>> {
        let sprite = self.try_get_sprite(sprite)?;
        self.try_get_object(sprite)
    }

    /// Creates an object with it's initial sprite being what is in the
    /// [SpriteBorrow]. Panics if there are no objects left. A [SpriteBorrow] is
    /// created using the [ObjectController::sprite] function.
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
    ///
    /// # fn foo(gba: &mut agb::Gba) {
    /// # let object_controller = gba.display.object.get();
    /// let emu = object_controller.object(object_controller.sprite(EMU_WALK.animation_sprite(0)));
    /// # }
    /// ```
    #[must_use]
    pub fn object(&self, sprite: SpriteBorrow) -> Object {
        self.try_get_object(sprite).expect("No object available")
    }

    /// Creates an object with it's initial sprite being what is in the
    /// [SpriteBorrow]. A [SpriteBorrow] is created using the
    /// [ObjectController::try_get_sprite] function.
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
    ///
    /// # fn foo(gba: &mut agb::Gba) {
    /// # let object_controller = gba.display.object.get();
    /// let emu = object_controller.try_get_object(
    ///     object_controller.sprite(EMU_WALK.animation_sprite(0))
    /// ).expect("the object should be allocatable");
    /// # }
    /// ```
    #[must_use]
    pub fn try_get_object(&self, sprite: SpriteBorrow) -> Option<Object> {
        let mut s = unsafe { self.inner.borrow_mut() };

        let mut attrs = Attributes::new();

        attrs.a2.set_tile_index(sprite.sprite.0.location.0 as u16);
        let shape_size = sprite.sprite.0.size.shape_size();
        attrs
            .a2
            .set_palete_bank((sprite.sprite.0.palette.0.location.0) as u8);
        attrs.a0.set_shape(shape_size.0);
        attrs.a1a.set_size(shape_size.1);
        attrs.a1s.set_size(shape_size.1);

        let index = s.free_object.pop()?;

        s.shadow_oam[index as usize] = Some(ObjectInner {
            attrs,
            z: 0,
            previous_sprite: sprite.clone(),
            destroy: false,
            sprite,
        });

        let loan = Loan {
            index: index as u8,
            controller: self.inner,
        };

        s.update_z_ordering();

        Some(Object { loan })
    }

    /// Creates a [SpriteBorrow] from the given sprite, panics if the sprite
    /// could not be allocated. This will reuse an existing copy of the sprite
    /// in vram if possible.
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
    ///
    /// # fn foo(gba: &mut agb::Gba) {
    /// # let object_controller = gba.display.object.get();
    /// let sprite = object_controller.sprite(EMU_WALK.animation_sprite(0));
    /// # }
    /// ```
    #[must_use]
    pub fn sprite(&self, sprite: &'static Sprite) -> SpriteBorrow {
        self.try_get_sprite(sprite)
            .expect("No slot for sprite available")
    }

    /// Creates a [SpriteBorrow] from the given sprite. This will reuse an
    /// existing copy of the sprite in vram if possible.
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
    ///
    /// # fn foo(gba: &mut agb::Gba) {
    /// # let object_controller = gba.display.object.get();
    /// let sprite = object_controller.try_get_sprite(
    ///     EMU_WALK.animation_sprite(0)
    /// ).expect("the sprite should be allocatable");
    /// # }
    /// ```
    #[must_use]
    pub fn try_get_sprite(&self, sprite: &'static Sprite) -> Option<SpriteBorrow> {
        unsafe { self.inner.borrow_mut() }
            .sprite_controller
            .try_get_sprite(sprite)
    }
}

impl<'a> Object<'a> {
    #[inline(always)]
    unsafe fn object_inner(&self) -> impl DerefMut<Target = ObjectInner> + 'a {
        #[cfg(debug_assertions)]
        {
            core::cell::RefMut::map(self.loan.controller.borrow_cell_ref(), |s| {
                s.shadow_oam[self.loan.index as usize]
                    .as_mut()
                    .unwrap_unchecked()
            })
        }
        #[cfg(not(debug_assertions))]
        {
            self.loan.controller.borrow_direct().shadow_oam[self.loan.index as usize]
                .as_mut()
                .unwrap_unchecked()
        }
    }

    /// Swaps out the current sprite. This handles changing of size, palette,
    /// etc. No change will be seen until [ObjectController::commit] is called.
    pub fn set_sprite(&'_ mut self, sprite: SpriteBorrow) {
        let mut object_inner = unsafe { self.object_inner() };
        object_inner
            .attrs
            .a2
            .set_tile_index(sprite.sprite.0.location.0 as u16);
        let shape_size = sprite.sprite.0.size.shape_size();
        object_inner
            .attrs
            .a2
            .set_palete_bank(sprite.sprite.0.palette.0.location.0 as u8);
        object_inner.attrs.a0.set_shape(shape_size.0);
        object_inner.attrs.a1a.set_size(shape_size.1);
        object_inner.attrs.a1s.set_size(shape_size.1);

        object_inner.sprite = sprite;
    }

    /// Shows the sprite. No change will be seen until
    /// [ObjectController::commit] is called.
    pub fn show(&mut self) -> &mut Self {
        {
            let mut object_inner = unsafe { self.object_inner() };
            object_inner.attrs.a0.set_object_mode(ObjectMode::Normal);
        }

        self
    }

    /// Controls whether the sprite is flipped horizontally, for example useful
    /// for reusing the same sprite for the left and right walking directions.
    /// No change will be seen until [ObjectController::commit] is called.
    pub fn set_hflip(&mut self, flip: bool) -> &mut Self {
        {
            let mut object_inner = unsafe { self.object_inner() };
            object_inner.attrs.a1s.set_horizontal_flip(flip);
        }
        self
    }

    /// Controls whether the sprite is flipped vertically, for example useful
    /// for reusing the same sprite for the up and down walking directions. No
    /// change will be seen until [ObjectController::commit] is called.
    pub fn set_vflip(&mut self, flip: bool) -> &mut Self {
        {
            let mut object_inner = unsafe { self.object_inner() };
            object_inner.attrs.a1s.set_vertical_flip(flip);
        }
        self
    }

    /// Sets the x position of the object. The coordinate refers to the top-left
    /// corner of the sprite. No change will be seen until
    /// [ObjectController::commit] is called.
    pub fn set_x(&mut self, x: u16) -> &mut Self {
        {
            let mut object_inner = unsafe { self.object_inner() };
            object_inner.attrs.a1a.set_x(x.rem_euclid(1 << 9) as u16);
            object_inner.attrs.a1s.set_x(x.rem_euclid(1 << 9) as u16);
        }
        self
    }

    /// Sets the z priority of the sprite. Higher priority will be dislayed
    /// above background layers with lower priorities. No change will be seen
    /// until [ObjectController::commit] is called.
    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        {
            let mut object_inner = unsafe { self.object_inner() };
            object_inner.attrs.a2.set_priority(priority);
        }
        self
    }

    /// Hides the object. No change will be seen until
    /// [ObjectController::commit] is called.
    pub fn hide(&mut self) -> &mut Self {
        {
            let mut object_inner = unsafe { self.object_inner() };
            object_inner.attrs.a0.set_object_mode(ObjectMode::Disabled);
        }
        self
    }

    /// Sets the y position of the sprite. The coordinate refers to the top-left
    /// corner of the sprite. No change will be seen until
    /// [ObjectController::commit] is called.
    pub fn set_y(&mut self, y: u16) -> &mut Self {
        {
            let mut object_inner = unsafe { self.object_inner() };
            object_inner.attrs.a0.set_y(y as u8);
        }

        self
    }

    /// Sets the z position of the sprite, this controls which sprites are above
    /// eachother. No change will be seen until [ObjectController::commit] is
    /// called.
    pub fn set_z(&mut self, z: i32) -> &mut Self {
        {
            let mut object_inner = unsafe { self.object_inner() };
            object_inner.z = z;
        }
        unsafe { self.loan.controller.borrow_mut().update_z_ordering() };

        self
    }

    /// Sets the position of the sprite using a [Vector2D]. The coordinate
    /// refers to the top-left corner of the sprite. No change will be seen
    /// until [ObjectController::commit] is called.
    pub fn set_position(&mut self, position: Vector2D<i32>) -> &mut Self {
        {
            let mut object_inner = unsafe { self.object_inner() };
            object_inner.attrs.a0.set_y(position.y as u8);
            object_inner
                .attrs
                .a1a
                .set_x(position.x.rem_euclid(1 << 9) as u16);
            object_inner
                .attrs
                .a1s
                .set_x(position.x.rem_euclid(1 << 9) as u16);
        }
        self
    }
}

/// The Sprite Id is a thin wrapper around the pointer to the sprite in
/// rom and is therefore a unique identifier to a sprite
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SpriteId(usize);

/// The palette id is a thin wrapper around the pointer to the palette in rom
/// and is therefore a unique reference to a palette
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct PaletteId(usize);

impl Palette16 {
    fn id(&'static self) -> PaletteId {
        PaletteId(self as *const _ as usize)
    }
    const fn layout() -> Layout {
        Layout::new::<Self>()
    }
}

impl Sprite {
    fn id(&'static self) -> SpriteId {
        SpriteId(self as *const _ as usize)
    }
    fn layout(&self) -> Layout {
        Layout::from_size_align(self.size.number_of_tiles() * BYTES_PER_TILE_4BPP, 8).unwrap()
    }
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
    /// The size of the sprite in it's form that is displayable on the GBA.
    pub const fn size(&self) -> Size {
        self.size
    }
}

impl SpriteControllerInner {
    fn try_get_sprite(&mut self, sprite: &'static Sprite) -> Option<SpriteBorrow> {
        let id = sprite.id();
        if let Some(storage) = self.static_sprite_map.get_mut(&id) {
            if let Some(strong) = storage.upgrade() {
                return Some(SpriteBorrow {
                    sprite: SpriteVram(strong),
                });
            }
        }

        // layout is non zero sized, so this is safe to call

        let palette_location = self.palette(sprite.palette);
        let palette_location = match palette_location {
            Some(a) => a,
            None => {
                return None;
            }
        };

        let dest = unsafe { SPRITE_ALLOCATOR.alloc(sprite.layout())? };

        unsafe {
            dma::dma_copy16(
                sprite.data.as_ptr().cast(),
                dest.as_ptr().cast(),
                sprite.data.len() / 2,
            );
        }

        let sprite = SpriteVram(Rc::new(SpriteArena {
            location: Location::from_sprite_ptr(dest),
            size: sprite.size(),
            palette: palette_location,
        }));

        self.static_sprite_map.insert(id, Rc::downgrade(&sprite.0));

        Some(SpriteBorrow { sprite })
    }

    /// Cleans up weak references to sprites and palettes no longer in vram
    fn gc(&mut self) {
        self.static_palette_map.retain(|_, v| v.strong_count() != 0);
        self.static_sprite_map.retain(|_, v| v.strong_count() != 0);
    }

    fn new() -> Self {
        Default::default()
    }
    fn palette(&mut self, palette: &'static Palette16) -> Option<PaletteVram> {
        let id = palette.id();
        if let Some(storage) = self.static_palette_map.get(&id) {
            if let Some(up) = storage.upgrade() {
                return Some(PaletteVram(up));
            }
        }

        let palette_vram = PaletteVram::new(palette)?;

        self.static_palette_map
            .insert(id, Rc::downgrade(&palette_vram.0));

        Some(palette_vram)
    }
}

#[derive(BitfieldSpecifier, Clone, Copy)]
enum ObjectMode {
    Normal,
    Affine,
    Disabled,
    AffineDouble,
}

#[derive(BitfieldSpecifier, Clone, Copy)]
#[bits = 2]
enum GraphicsMode {
    Normal,
    AlphaBlending,
    Window,
}

#[derive(BitfieldSpecifier, Clone, Copy)]
enum ColourMode {
    Four,
    Eight,
}

// this mod is not public, so the internal parts don't need documenting.
#[allow(dead_code)]
mod attributes {
    use super::*;
    #[bitfield]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(super) struct ObjectAttribute0 {
        pub y: B8,
        pub object_mode: ObjectMode,
        pub graphics_mode: GraphicsMode,
        pub mosaic: bool,
        pub colour_mode: ColourMode,
        pub shape: B2,
    }

    #[bitfield]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(super) struct ObjectAttribute1Standard {
        pub x: B9,
        #[skip]
        __: B3,
        pub horizontal_flip: bool,
        pub vertical_flip: bool,
        pub size: B2,
    }

    #[bitfield]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(super) struct ObjectAttribute1Affine {
        pub x: B9,
        pub affine_index: B5,
        pub size: B2,
    }

    #[bitfield]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(super) struct ObjectAttribute2 {
        pub tile_index: B10,
        pub priority: Priority,
        pub palete_bank: B4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test_case]
    fn size_of_ObjectControllerReference(_: &mut crate::Gba) {
        if !cfg!(debug_assertions) {
            assert_eq!(size_of::<ObjectControllerReference>(), 0);
        }
    }

    #[test_case]
    fn object_usage(gba: &mut crate::Gba) {
        const GRAPHICS: &Graphics = include_aseprite!(
            "../examples/the-purple-night/gfx/objects.aseprite",
            "../examples/the-purple-night/gfx/boss.aseprite"
        );

        const BOSS: &Tag = GRAPHICS.tags().get("Boss");
        const EMU: &Tag = GRAPHICS.tags().get("emu - idle");

        let object = gba.display.object.get();

        {
            let mut objects: Vec<_> = alloc::vec![
                object.object(object.sprite(BOSS.sprite(0))),
                object.object(object.sprite(EMU.sprite(0))),
            ]
            .into_iter()
            .map(Some)
            .collect();

            object.commit();

            let x = objects[0].as_mut().unwrap();
            x.set_hflip(true);
            x.set_vflip(true);
            x.set_position((1, 1).into());
            x.set_z(100);
            x.set_sprite(object.sprite(BOSS.sprite(2)));

            object.commit();
        }

        object.commit();
    }
}
