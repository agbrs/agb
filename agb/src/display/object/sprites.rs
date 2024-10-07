mod sprite;
mod sprite_allocator;

const BYTES_PER_TILE_4BPP: usize = 32;
const BYTES_PER_TILE_8BPP: usize = 16;

pub use sprite::{include_aseprite, Graphics, PaletteMulti, Size, Sprite, Tag, TagMap};
pub use sprite_allocator::{
    DynamicSprite, IntoSpritePaletteVram, IntoSpriteVram, PaletteVram, PaletteVramInterface,
    PaletteVramMulti, PaletteVramSingle, SpriteVram, SPRITE_LOADER,
};
