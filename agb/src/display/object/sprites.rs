mod sprite;
mod sprite_allocator;

const BYTES_PER_TILE_4BPP: usize = 32;
const BYTES_PER_TILE_8BPP: usize = 16;

pub use sprite::{PaletteMulti, Size, Sprite, Tag, include_aseprite};
pub use sprite_allocator::{
    DynamicSprite16, DynamicSprite256, PaletteVram, PaletteVramMulti, PaletteVramSingle, SpriteVram,
};
