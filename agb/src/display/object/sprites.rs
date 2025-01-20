mod sprite;
mod sprite_allocator;

const BYTES_PER_TILE_4BPP: usize = 32;
const BYTES_PER_TILE_8BPP: usize = 16;

pub use sprite::{include_aseprite, Graphics, MultiPalette, Size, Sprite, Tag, TagMap};
pub use sprite_allocator::{
    DynamicSprite, MultiPaletteVram, PaletteVram, PaletteVramInterface, SinglePaletteVram,
    SpriteLoader, SpriteVram,
};
