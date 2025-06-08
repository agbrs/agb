pub use palette::{PaletteVram, PaletteVramMulti, PaletteVramSingle};
use sprite::SpriteVramInner;

pub use dynamic::{DynamicSprite16, DynamicSprite256};
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
    pub(crate) const fn new() -> Self {
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

pub struct SpriteLoader(SyncUnsafeCell<SpriteLoaderInner>);

impl SpriteLoader {
    unsafe fn with<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&mut SpriteLoaderInner) -> U,
    {
        unsafe { f(&mut *self.0.get()) }
    }

    pub unsafe fn sprite(&self, sprite: &'static Sprite) -> Result<SpriteVram, LoaderError> {
        unsafe { self.with(|x| x.try_allocate_sprite(sprite)) }
    }

    pub unsafe fn palette(&self, palette: Palette) -> Result<PaletteVram, LoaderError> {
        unsafe { self.with(|x| x.try_allocate_palette(palette)) }
    }
}

pub(crate) unsafe fn garbage_collect_sprite_loader() {
    unsafe {
        SPRITE_LOADER.with(|x| {
            x.garbage_collect_sprites();
            x.garbage_collect_palettes();
        });
    }
}

pub static SPRITE_LOADER: SpriteLoader =
    SpriteLoader(SyncUnsafeCell::new(SpriteLoaderInner::new()));

impl From<&'static Palette16> for PaletteVram {
    fn from(value: &'static Palette16) -> Self {
        PaletteVram::new_single(value).expect("out of palette space")
    }
}

impl From<&'static Palette16> for PaletteVramSingle {
    fn from(value: &'static Palette16) -> Self {
        PaletteVramSingle::new(value)
    }
}

impl TryFrom<PaletteVram> for PaletteVramSingle {
    type Error = PaletteVram;

    fn try_from(value: PaletteVram) -> Result<Self, Self::Error> {
        value.single()
    }
}

impl TryFrom<PaletteVram> for PaletteVramMulti {
    type Error = PaletteVram;

    fn try_from(value: PaletteVram) -> Result<Self, Self::Error> {
        value.multi()
    }
}

impl From<PaletteVramSingle> for PaletteVram {
    fn from(value: PaletteVramSingle) -> Self {
        value.palette()
    }
}

impl From<PaletteVramMulti> for PaletteVram {
    fn from(value: PaletteVramMulti) -> Self {
        value.palette()
    }
}

impl From<&'static Sprite> for SpriteVram {
    fn from(value: &'static Sprite) -> Self {
        unsafe { SPRITE_LOADER.sprite(value) }.expect("have space for sprites")
    }
}
