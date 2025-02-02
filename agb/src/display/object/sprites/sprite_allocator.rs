use core::mem::MaybeUninit;

pub use palette::{PaletteVram, PaletteVramMulti, PaletteVramSingle};
use sprite::SpriteVramInner;

pub use dynamic::{DynamicSprite, PaletteVramInterface};
pub use sprite::SpriteVram;

use crate::{display::palette16::Palette16, hash_map::HashMap, util::SyncUnsafeCell};

use super::sprite::{Palette, PaletteMulti, Sprite};

mod dynamic;
mod palette;
mod sprite;

/// The Sprite Id is a thin wrapper around the pointer to the sprite in
/// rom and is therefore a unique identifier to a sprite
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SpriteId(usize);

impl SpriteId {
    fn new(sprite: &'static Sprite) -> Self {
        Self(sprite as *const _ as usize)
    }
}

/// The palette id is a thin wrapper around the pointer to the palette in rom
/// and is therefore a unique reference to a palette
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct PaletteId(usize);

impl PaletteId {
    fn new_single(palette: &'static Palette16) -> Self {
        Self(palette as *const _ as usize)
    }

    fn new_multi(palette: &'static PaletteMulti) -> Self {
        Self(palette as *const _ as usize)
    }

    fn new(palette: Palette) -> Self {
        match palette {
            Palette::Single(palette16) => Self::new_single(palette16),
            Palette::Multi(palette_multi) => Self::new_multi(palette_multi),
        }
    }
}

/// This holds loading of static sprites and palettes.
struct SpriteLoaderInner {
    palettes: HashMap<PaletteId, PaletteVram>,
    sprites: HashMap<SpriteId, SpriteVramInner>,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoaderError {
    SpriteFull,
    PaletteFull,
}

impl SpriteLoaderInner {
    pub(crate) fn new() -> Self {
        Self {
            palettes: HashMap::new(),
            sprites: HashMap::new(),
        }
    }

    fn garbage_collect_sprites(&mut self) {
        self.sprites.retain(|_, v| v.strong_count() > 1);
    }

    fn garbage_collect_palettes(&mut self) {
        self.palettes.retain(|_, v| v.strong_count() > 1);
    }

    fn try_allocate_palette_inner(&mut self, palette: Palette) -> Result<PaletteVram, LoaderError> {
        match self.palettes.entry(PaletteId::new(palette)) {
            agb_hashmap::Entry::Occupied(occupied_entry) => Ok(occupied_entry.get().clone()),
            agb_hashmap::Entry::Vacant(vacant_entry) => {
                let palette = match palette {
                    Palette::Single(palette16) => PaletteVram::new_single(palette16),
                    Palette::Multi(palette_multi) => PaletteVram::new_multi(palette_multi),
                }?;
                vacant_entry.insert(palette.clone());
                Ok(palette)
            }
        }
    }

    fn try_allocate_palette(&mut self, palette: Palette) -> Result<PaletteVram, LoaderError> {
        if let Ok(palette) = self.try_allocate_palette_inner(palette) {
            return Ok(palette);
        }
        self.garbage_collect_palettes();
        self.try_allocate_palette_inner(palette)
    }

    fn try_allocate_sprite_inner(
        &mut self,
        sprite: &'static Sprite,
    ) -> Result<SpriteVramInner, LoaderError> {
        match self.sprites.entry(SpriteId::new(sprite)) {
            agb_hashmap::Entry::Occupied(occupied_entry) => {
                let sprite = occupied_entry.get();
                Ok(sprite.clone())
            }
            agb_hashmap::Entry::Vacant(vacant_entry) => {
                let sprite = SpriteVramInner::new_from_sprite(sprite)?;
                vacant_entry.insert(sprite.clone());
                Ok(sprite)
            }
        }
    }

    fn try_allocate_sprite(&mut self, sprite: &'static Sprite) -> Result<SpriteVram, LoaderError> {
        let palette = self.try_allocate_palette(sprite.palette)?;
        let sprite = match self.try_allocate_sprite_inner(sprite) {
            Ok(sprite) => sprite,
            Err(_) => {
                self.garbage_collect_sprites();
                self.try_allocate_sprite_inner(sprite)?
            }
        };

        Ok(SpriteVram::new(sprite, palette))
    }
}

pub struct SpriteLoader(SyncUnsafeCell<MaybeUninit<SpriteLoaderInner>>);

impl SpriteLoader {
    pub unsafe fn init(&self) {
        unsafe {
            (*self.0.get()).write(SpriteLoaderInner::new());
        }
    }

    unsafe fn with<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&mut SpriteLoaderInner) -> U,
    {
        unsafe { f((*self.0.get()).assume_init_mut()) }
    }

    pub unsafe fn sprite(&self, sprite: &'static Sprite) -> Result<SpriteVram, LoaderError> {
        unsafe { self.with(|x| x.try_allocate_sprite(sprite)) }
    }

    pub unsafe fn palette(&self, palette: Palette) -> Result<PaletteVram, LoaderError> {
        unsafe { self.with(|x| x.try_allocate_palette(palette)) }
    }
}

pub static SPRITE_LOADER: SpriteLoader = SpriteLoader(SyncUnsafeCell::new(MaybeUninit::uninit()));

pub trait IntoSpritePaletteVram: Sized {
    fn into(self) -> PaletteVram {
        self.try_into().expect("could not create palette in vram")
    }
    fn try_into(self) -> Result<PaletteVram, LoaderError>;
}

pub trait IntoSpriteVram: Sized {
    fn into(self) -> SpriteVram {
        self.try_into().expect("could not create sprite in vram")
    }
    fn try_into(self) -> Result<SpriteVram, LoaderError>;
}

impl IntoSpritePaletteVram for &'static Palette16 {
    fn try_into(self) -> Result<PaletteVram, LoaderError> {
        unsafe { SPRITE_LOADER.palette(Palette::Single(self)) }
    }
}
impl IntoSpritePaletteVram for PaletteVram {
    fn try_into(self) -> Result<PaletteVram, LoaderError> {
        Ok(self)
    }
}
impl IntoSpritePaletteVram for &PaletteVram {
    fn try_into(self) -> Result<PaletteVram, LoaderError> {
        Ok(self.clone())
    }
}
impl IntoSpritePaletteVram for &'static PaletteMulti {
    fn try_into(self) -> Result<PaletteVram, LoaderError> {
        unsafe { SPRITE_LOADER.palette(Palette::Multi(self)) }
    }
}
impl IntoSpriteVram for &'static Sprite {
    fn try_into(self) -> Result<SpriteVram, LoaderError> {
        unsafe { SPRITE_LOADER.sprite(self) }
    }
}
impl IntoSpriteVram for SpriteVram {
    fn try_into(self) -> Result<SpriteVram, LoaderError> {
        Ok(self)
    }
}
