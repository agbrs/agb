mod sprite;
mod sprite_allocator;

const BYTES_PER_TILE_4BPP: usize = 32;

pub use sprite::{include_aseprite, Graphics, Size, Sprite, Tag, TagMap};
pub use sprite_allocator::{DynamicSprite, SpriteLoader, SpriteVram};
