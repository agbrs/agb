use alloc::vec::Vec;
use core::alloc::Layout;

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
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
use crate::interrupt::free;

use bare_metal::Mutex;
use core::cell::RefCell;

use attributes::*;

static mut OBJECT_CONTROLLER: MaybeUninit<ObjectControllerStatic> = MaybeUninit::uninit();

unsafe fn init_object_controller() {
    OBJECT_CONTROLLER.write(ObjectControllerStatic::new());
}

unsafe fn uninit_object_controller() {
    OBJECT_CONTROLLER.assume_init_drop()
}

struct ObjectControllerRef {}

impl Deref for ObjectControllerRef {
    type Target = ObjectControllerStatic;
    fn deref(&self) -> &'static ObjectControllerStatic {
        unsafe { OBJECT_CONTROLLER.assume_init_ref() }
    }
}

impl DerefMut for ObjectControllerRef {
    fn deref_mut(&mut self) -> &'static mut ObjectControllerStatic {
        unsafe { OBJECT_CONTROLLER.assume_init_mut() }
    }
}

#[cfg(debug_assertions)]
static OBJECT_REFS_CURRENT: Mutex<RefCell<i32>> = Mutex::new(RefCell::new(0));

impl ObjectControllerRef {
    fn new() -> Self {
        #[cfg(debug_assertions)]
        {
            let a = free(|c| {
                let mut b = OBJECT_REFS_CURRENT.borrow(*c).borrow_mut();
                let a = *b;
                *b += 1;
                a
            });
            assert_eq!(a, 0);
        }

        Self {}
    }

    unsafe fn very_unsafe_borrow(&self) -> &'static mut ObjectControllerStatic {
        OBJECT_CONTROLLER.assume_init_mut()
    }
}

#[cfg(debug_assertions)]
impl Drop for ObjectControllerRef {
    fn drop(&mut self) {
        free(|c| {
            let mut b = OBJECT_REFS_CURRENT.borrow(*c).borrow_mut();
            *b -= 1;
        })
    }
}

unsafe fn get_object_controller(_r: &ObjectControllerReference) -> ObjectControllerRef {
    ObjectControllerRef::new()
}

/// Include this type if you call `get_object_controller` in impl block. This
/// helps you use the right lifetimes and doesn't impl Sync (using from two
/// "threads" without syncronisation is not safe), but sending to another
/// "thread" is safe.
type ObjectControllerReference<'a> = PhantomData<&'a UnsafeCell<()>>;

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

pub struct Sprite {
    palette: &'static Palette16,
    data: &'static [u8],
    size: Size,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[macro_export]
macro_rules! include_aseprite {
    ($($aseprite_path: expr),*) => {{
        use $crate::display::object::{Size, Sprite, Tag, TagMap, Graphics};
        use $crate::display::palette16::Palette16;

        $crate::include_aseprite_inner!($($aseprite_path),*);

        &Graphics::new(SPRITES, TAGS)
    }};
}

pub struct Graphics {
    sprites: &'static [Sprite],
    tag_map: &'static TagMap,
}

impl Graphics {
    pub const fn new(sprites: &'static [Sprite], tag_map: &'static TagMap) -> Self {
        Self { sprites, tag_map }
    }
    pub const fn tags(&self) -> &TagMap {
        self.tag_map
    }
    pub const fn sprites(&self) -> &[Sprite] {
        self.sprites
    }
}

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
    pub const fn new(tags: &'static [(&'static str, Tag)]) -> TagMap {
        Self { tags }
    }
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
    pub const fn get(&'static self, tag: &str) -> &'static Tag {
        let t = self.try_get(tag);
        match t {
            Some(t) => t,
            None => panic!("The requested tag does not exist"),
        }
    }
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

pub struct Tag {
    sprites: *const Sprite,
    len: usize,
    direction: Direction,
}

impl Tag {
    pub fn sprites(&self) -> &'static [Sprite] {
        unsafe { slice::from_raw_parts(self.sprites, self.len) }
    }

    pub fn sprite(&self, idx: usize) -> &'static Sprite {
        &self.sprites()[idx]
    }

    #[inline]
    pub fn animation_sprite(&self, idx: usize) -> &'static Sprite {
        let len_sub_1 = self.len - 1;
        match self.direction {
            Direction::Forward => self.sprite(idx % self.len),
            Direction::Backward => self.sprite(len_sub_1 - (idx % self.len)),
            Direction::Pingpong => self.sprite(
                (((idx + len_sub_1) % (len_sub_1 * 2)) as isize - len_sub_1 as isize).abs()
                    as usize,
            ),
        }
    }

    #[doc(hidden)]
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

pub struct SpriteBorrow<'a> {
    id: SpriteId,
    sprite_location: u16,
    palette_location: u16,
    phantom: ObjectControllerReference<'a>,
}

#[derive(Clone, Copy)]
struct Storage {
    location: u16,
    count: u16,
}

impl Storage {
    fn from_sprite_ptr(d: NonNull<u8>) -> Self {
        Self {
            location: (((d.as_ptr() as usize) - TILE_SPRITE) / BYTES_PER_TILE_4BPP) as u16,
            count: 1,
        }
    }
    fn from_palette_ptr(d: NonNull<u8>) -> Self {
        Self {
            location: ((d.as_ptr() as usize - PALETTE_SPRITE) / Palette16::layout().size()) as u16,
            count: 1,
        }
    }
    fn as_palette_ptr(&self) -> *mut u8 {
        (self.location as usize * Palette16::layout().size() + PALETTE_SPRITE) as *mut u8
    }
    fn as_sprite_ptr(&self) -> *mut u8 {
        (self.location as usize * BYTES_PER_TILE_4BPP + TILE_SPRITE) as *mut u8
    }
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

pub struct Object<'a> {
    loan: Loan<'a>,
}

struct SpriteControllerInner {
    palette: HashMap<PaletteId, Storage>,
    sprite: HashMap<SpriteId, Storage>,
}

struct Loan<'a> {
    index: u8,
    phantom: ObjectControllerReference<'a>,
}

impl Drop for Loan<'_> {
    fn drop(&mut self) {
        let mut s = unsafe { get_object_controller(&self.phantom) };

        unsafe {
            s.shadow_oam[self.index as usize]
                .as_mut()
                .unwrap_unchecked()
                .destroy = true
        };
    }
}

struct ObjectInner {
    attrs: Attributes,
    sprite: SpriteBorrow<'static>,
    previous_sprite: SpriteBorrow<'static>,
    destroy: bool,
    z: i32,
}

struct ObjectControllerStatic {
    free_affine_matricies: Vec<u8>,
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
            free_affine_matricies: (0..32).collect(),
            sprite_controller: SpriteControllerInner::new(),
        }
    }

    fn update_z_ordering(&mut self) {
        let shadow_oam = &self.shadow_oam;
        self.z_order.sort_by_key(|&a| {
            shadow_oam[a as usize]
                .as_ref()
                .map(|s| s.z)
                .unwrap_or(i32::MAX)
        });
    }
}

pub struct ObjectController {
    phantom: ObjectControllerReference<'static>,
}

impl Drop for ObjectController {
    fn drop(&mut self) {
        unsafe {
            uninit_object_controller();
        }
    }
}

const HIDDEN_VALUE: u16 = 0b10 << 8;

impl ObjectController {
    pub fn commit(&self) {
        let mut s = unsafe { get_object_controller(&self.phantom) };

        let s = &mut *s;

        for (i, &z) in s.z_order.iter().enumerate() {
            if let Some(o) = &mut s.shadow_oam[z as usize] {
                if o.destroy {
                    s.free_object.push(z);

                    unsafe {
                        (OBJECT_ATTRIBUTE_MEMORY as *mut u16)
                            .add((i as usize) * 4)
                            .write_volatile(HIDDEN_VALUE)
                    }

                    let a = unsafe { s.shadow_oam[z as usize].take().unwrap_unchecked() };
                    a.previous_sprite.drop(&mut s.sprite_controller);
                    a.sprite.drop(&mut s.sprite_controller);
                } else {
                    o.attrs.commit(i);

                    let mut a = o.sprite.clone(&mut s.sprite_controller);
                    core::mem::swap(&mut o.previous_sprite, &mut a);
                    a.drop(&mut s.sprite_controller);
                }
            } else {
                unsafe {
                    (OBJECT_ATTRIBUTE_MEMORY as *mut u16)
                        .add(i * 4)
                        .write_volatile(HIDDEN_VALUE)
                }
            }
        }
    }

    pub(crate) fn new() -> Self {
        DISPLAY_CONTROL.set_bits(1, 1, 0x6);
        DISPLAY_CONTROL.set_bits(1, 1, 0xC);
        DISPLAY_CONTROL.set_bits(0, 1, 0x7);

        for i in 0..128 {
            unsafe {
                (OBJECT_ATTRIBUTE_MEMORY as *mut u16)
                    .add(i * 4)
                    .write_volatile(HIDDEN_VALUE)
            }
        }

        unsafe { init_object_controller() };
        Self {
            phantom: PhantomData,
        }
    }

    pub fn object<'a>(&'a self, sprite: SpriteBorrow<'a>) -> Object<'a> {
        self.try_get_object(sprite).expect("No object available")
    }

    pub fn try_get_object<'a>(&'a self, sprite: SpriteBorrow<'a>) -> Option<Object<'a>> {
        let mut s = unsafe { get_object_controller(&self.phantom) };

        let mut attrs = Attributes::new();

        attrs.a2.set_tile_index(sprite.sprite_location);
        let shape_size = sprite.id.sprite().size.shape_size();
        attrs.a2.set_palete_bank(sprite.palette_location as u8);
        attrs.a0.set_shape(shape_size.0);
        attrs.a1a.set_size(shape_size.1);
        attrs.a1s.set_size(shape_size.1);

        let index = s.free_object.pop()?;

        let new_sprite: SpriteBorrow<'static> = unsafe { core::mem::transmute(sprite) };

        s.shadow_oam[index as usize] = Some(ObjectInner {
            attrs,
            z: 0,
            previous_sprite: new_sprite.clone(&mut s.sprite_controller),
            destroy: false,
            sprite: new_sprite,
        });

        let loan = Loan {
            index: index as u8,
            phantom: PhantomData,
        };

        s.update_z_ordering();

        Some(Object { loan })
    }

    pub fn sprite(&self, sprite: &'static Sprite) -> SpriteBorrow {
        self.try_get_sprite(sprite)
            .expect("No slot for sprite available")
    }

    pub fn try_get_sprite(&self, sprite: &'static Sprite) -> Option<SpriteBorrow> {
        let s = unsafe { get_object_controller(&self.phantom) };
        unsafe {
            s.very_unsafe_borrow()
                .sprite_controller
                .try_get_sprite(sprite)
        }
    }
}

impl<'a> Object<'a> {
    #[inline(always)]
    unsafe fn object_inner(&mut self) -> &mut ObjectInner {
        let s = get_object_controller(&self.loan.phantom);
        s.very_unsafe_borrow().shadow_oam[self.loan.index as usize]
            .as_mut()
            .unwrap_unchecked()
    }

    pub fn set_sprite(&'_ mut self, sprite: SpriteBorrow<'a>) {
        let object_inner = unsafe { self.object_inner() };
        object_inner.attrs.a2.set_tile_index(sprite.sprite_location);
        let shape_size = sprite.id.sprite().size.shape_size();
        object_inner
            .attrs
            .a2
            .set_palete_bank(sprite.palette_location as u8);
        object_inner.attrs.a0.set_shape(shape_size.0);
        object_inner.attrs.a1a.set_size(shape_size.1);
        object_inner.attrs.a1s.set_size(shape_size.1);
        object_inner.sprite = unsafe { core::mem::transmute(sprite) };
    }

    pub fn show(&mut self) -> &mut Self {
        let object_inner = unsafe { self.object_inner() };
        object_inner.attrs.a0.set_object_mode(ObjectMode::Normal);

        self
    }

    pub fn set_hflip(&mut self, flip: bool) -> &mut Self {
        let object_inner = unsafe { self.object_inner() };
        object_inner.attrs.a1s.set_horizontal_flip(flip);
        self
    }

    pub fn set_vflip(&mut self, flip: bool) -> &mut Self {
        let object_inner = unsafe { self.object_inner() };
        object_inner.attrs.a1s.set_vertical_flip(flip);
        self
    }

    pub fn set_x(&mut self, x: u16) -> &mut Self {
        let object_inner = unsafe { self.object_inner() };
        object_inner.attrs.a1a.set_x(x.rem_euclid(1 << 9) as u16);
        object_inner.attrs.a1s.set_x(x.rem_euclid(1 << 9) as u16);
        self
    }

    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        let object_inner = unsafe { self.object_inner() };
        object_inner.attrs.a2.set_priority(priority);
        self
    }

    pub fn hide(&mut self) -> &mut Self {
        let object_inner = unsafe { self.object_inner() };
        object_inner.attrs.a0.set_object_mode(ObjectMode::Disabled);
        self
    }

    pub fn set_y(&mut self, y: u16) -> &mut Self {
        let object_inner = unsafe { self.object_inner() };
        object_inner.attrs.a0.set_y(y as u8);

        self
    }

    pub fn set_z(&mut self, z: i32) -> &mut Self {
        let object_inner = unsafe { self.object_inner() };
        object_inner.z = z;
        unsafe {
            get_object_controller(&self.loan.phantom).update_z_ordering();
        }

        self
    }

    pub fn set_position(&mut self, position: Vector2D<i32>) -> &mut Self {
        let object_inner = unsafe { self.object_inner() };
        object_inner.attrs.a0.set_y(position.y as u8);
        object_inner
            .attrs
            .a1a
            .set_x(position.x.rem_euclid(1 << 9) as u16);
        object_inner
            .attrs
            .a1s
            .set_x(position.x.rem_euclid(1 << 9) as u16);
        self
    }
}

/// The Sprite Id is a thin wrapper around the pointer to the sprite in
/// rom and is therefore a unique identifier to a sprite
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SpriteId(usize);

impl SpriteId {
    fn sprite(self) -> &'static Sprite {
        // # Safety
        // This must be constructed using the id() of a sprite, so
        // they are always valid and always static
        unsafe { (self.0 as *const Sprite).as_ref().unwrap_unchecked() }
    }
}

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
    pub const fn new(palette: &'static Palette16, data: &'static [u8], size: Size) -> Self {
        Self {
            palette,
            data,
            size,
        }
    }
    pub const fn size(&self) -> Size {
        self.size
    }
}

impl SpriteControllerInner {
    fn try_get_sprite(&mut self, sprite: &'static Sprite) -> Option<SpriteBorrow> {
        let id = sprite.id();
        if let Some(storage) = self.sprite.get_mut(&id) {
            storage.count += 1;
            let location = storage.location;
            let palette_location = self.palette(sprite.palette).unwrap();
            Some(SpriteBorrow {
                id,
                palette_location,
                sprite_location: location,
                phantom: PhantomData,
            })
        } else {
            // layout is non zero sized, so this is safe to call

            let dest = unsafe { SPRITE_ALLOCATOR.alloc(sprite.layout())? };

            let palette_location = self.palette(sprite.palette);
            let palette_location = match palette_location {
                Some(a) => a,
                None => {
                    unsafe { SPRITE_ALLOCATOR.dealloc(dest.as_ptr(), sprite.layout()) }
                    return None;
                }
            };

            unsafe {
                dma::dma_copy16(
                    sprite.data.as_ptr().cast(),
                    dest.as_ptr().cast(),
                    sprite.data.len() / 2,
                );
            }

            let storage = Storage::from_sprite_ptr(dest);
            self.sprite.insert(id, storage);

            Some(SpriteBorrow {
                id,
                palette_location,
                sprite_location: storage.location,
                phantom: PhantomData,
            })
        }
    }
}

impl SpriteControllerInner {
    fn new() -> Self {
        Self {
            palette: HashMap::default(),
            sprite: HashMap::default(),
        }
    }
    fn palette(&mut self, palette: &'static Palette16) -> Option<u16> {
        let id = palette.id();
        if let Some(storage) = self.palette.get_mut(&id) {
            storage.count += 1;
            Some(storage.location)
        } else {
            let dest = unsafe { PALETTE_ALLOCATOR.alloc(Palette16::layout())? };

            unsafe {
                dma::dma_copy16(
                    palette.colours.as_ptr().cast(),
                    dest.as_ptr().cast(),
                    palette.colours.len(),
                );
            }

            let storage = Storage::from_palette_ptr(dest);
            self.palette.insert(id, storage);

            Some(storage.location)
        }
    }

    fn return_sprite(&mut self, sprite: &'static Sprite) {
        let storage = self.sprite.get_mut(&sprite.id());

        if let Some(storage) = storage {
            storage.count -= 1;

            if storage.count == 0 {
                unsafe { SPRITE_ALLOCATOR.dealloc(storage.as_sprite_ptr(), sprite.layout()) };
                self.sprite.remove(&sprite.id());
            }
        }

        self.return_palette(sprite.palette)
    }

    fn return_palette(&mut self, palette: &'static Palette16) {
        let id = palette.id();

        if let Some(storage) = self.palette.get_mut(&id) {
            storage.count -= 1;

            if storage.count == 0 {
                unsafe { PALETTE_ALLOCATOR.dealloc(storage.as_palette_ptr(), Palette16::layout()) };
                self.palette.remove(&id);
            }
        }
    }
}

impl<'a> Drop for SpriteBorrow<'a> {
    fn drop(&mut self) {
        let mut s = unsafe { get_object_controller(&self.phantom) };
        s.sprite_controller.return_sprite(self.id.sprite())
    }
}

impl<'a> SpriteBorrow<'a> {
    fn drop(self, s: &mut SpriteControllerInner) {
        s.return_sprite(self.id.sprite());
        core::mem::forget(self);
    }

    fn clone(&self, s: &mut SpriteControllerInner) -> Self {
        s.sprite.entry(self.id).and_modify(|a| a.count += 1);
        let _ = s.palette(self.id.sprite().palette).unwrap();
        Self {
            id: self.id,
            sprite_location: self.sprite_location,
            palette_location: self.palette_location,
            phantom: PhantomData,
        }
    }
}

impl<'a> Clone for SpriteBorrow<'a> {
    fn clone(&self) -> Self {
        let mut s = unsafe { get_object_controller(&self.phantom) };
        self.clone(&mut s.sprite_controller)
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
        assert_eq!(size_of::<ObjectControllerReference>(), 0);
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
}
